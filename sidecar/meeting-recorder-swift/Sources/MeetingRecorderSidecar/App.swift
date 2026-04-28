import AVFoundation
import Foundation
import ScreenCaptureKit

// MARK: - Logging (stderr only — stdout is reserved for protocol)

func log(_ msg: String) {
    fputs("[meeting-recorder] \(msg)\n", stderr)
    fflush(stderr)
}

// MARK: - Protocol emitters (line-delimited JSON to stdout)

func emit(_ obj: [String: Any]) {
    if JSONSerialization.isValidJSONObject(obj),
        let data = try? JSONSerialization.data(withJSONObject: obj),
        let s = String(data: data, encoding: .utf8)
    {
        print(s)
        fflush(stdout)
    }
}

func emitReady() { emit(["type": "ready"]) }
func emitStopped() { emit(["type": "stopped"]) }
func emitError(_ message: String) { emit(["type": "error", "message": message]) }
func emitPermissionDenied(kind: String, message: String) {
    emit(["type": "permission_denied", "kind": kind, "message": message])
}
func emitChunk(stream: String, wavPath: String, startedAtMs: Int, durationMs: Int, sampleRate: Int)
{
    emit([
        "type": "chunk",
        "stream": stream,
        "wav_path": wavPath,
        "started_at_ms": startedAtMs,
        "duration_ms": durationMs,
        "sample_rate": sampleRate,
    ])
}

// MARK: - CLI Args

struct Args {
    var outputDir: String = ""
    var chunkMs: Int = 15000
    var sampleRate: Int = 16000

    static func parse() -> Args {
        var args = Args()
        let argv = CommandLine.arguments
        var i = 1
        while i < argv.count {
            let key = argv[i]
            let next: () -> String? = {
                i += 1
                return i < argv.count ? argv[i] : nil
            }
            switch key {
            case "--output-dir":
                if let v = next() { args.outputDir = v }
            case "--chunk-ms":
                if let v = next(), let n = Int(v) { args.chunkMs = n }
            case "--sample-rate":
                if let v = next(), let n = Int(v) { args.sampleRate = n }
            default:
                log("unknown arg: \(key)")
            }
            i += 1
        }
        return args
    }
}

// MARK: - Endian helpers

extension UInt32 {
    var littleEndianData: Data {
        var le = self.littleEndian
        return Data(bytes: &le, count: 4)
    }
}
extension UInt16 {
    var littleEndianData: Data {
        var le = self.littleEndian
        return Data(bytes: &le, count: 2)
    }
}
extension Int16 {
    var littleEndianData: Data {
        var le = self.littleEndian
        return Data(bytes: &le, count: 2)
    }
}

// MARK: - WAV writer (16-bit PCM mono)

func writeWav(samples: [Float], sampleRate: Int, to path: String) throws {
    var data = Data()
    let dataSize = UInt32(samples.count * 2)
    let chunkSize = dataSize + 36

    data.append("RIFF".data(using: .ascii)!)
    data.append(chunkSize.littleEndianData)
    data.append("WAVE".data(using: .ascii)!)

    data.append("fmt ".data(using: .ascii)!)
    data.append(UInt32(16).littleEndianData)  // PCM fmt size
    data.append(UInt16(1).littleEndianData)  // PCM format
    data.append(UInt16(1).littleEndianData)  // mono
    data.append(UInt32(sampleRate).littleEndianData)
    data.append(UInt32(sampleRate * 2).littleEndianData)  // byte rate
    data.append(UInt16(2).littleEndianData)  // block align
    data.append(UInt16(16).littleEndianData)  // bits per sample

    data.append("data".data(using: .ascii)!)
    data.append(dataSize.littleEndianData)
    data.reserveCapacity(data.count + samples.count * 2)
    for s in samples {
        let clamped = max(-1.0, min(1.0, s))
        let i16 = Int16(clamped * 32767.0)
        data.append(i16.littleEndianData)
    }

    try data.write(to: URL(fileURLWithPath: path))
}

// MARK: - Audio buffer (thread-safe)

final class AudioRingBuffer {
    private var samples: [Float] = []
    private let lock = NSLock()
    private let maxCapacity: Int

    init(maxSeconds: Int = 600, sampleRate: Int = 16000) {
        self.maxCapacity = maxSeconds * sampleRate
    }

    func append(_ chunk: [Float]) {
        lock.lock()
        defer { lock.unlock() }
        samples.append(contentsOf: chunk)
        if samples.count > maxCapacity {
            samples.removeFirst(samples.count - maxCapacity)
        }
    }

    /// Drain up to `count` samples from the front. Returns the actual samples taken.
    func drain(upTo count: Int) -> [Float] {
        lock.lock()
        defer { lock.unlock() }
        let n = min(count, samples.count)
        let taken = Array(samples.prefix(n))
        samples.removeFirst(n)
        return taken
    }

    var count: Int {
        lock.lock()
        defer { lock.unlock() }
        return samples.count
    }
}

// MARK: - Audio format converter helper

/// Converts a CMSampleBuffer of audio to mono Float32 at target sample rate.
/// Returns the converted samples.
func convertSampleBufferToMonoFloat32(
    _ sampleBuffer: CMSampleBuffer,
    targetSampleRate: Double
) -> [Float]? {
    guard let formatDescription = CMSampleBufferGetFormatDescription(sampleBuffer),
        let asbd = CMAudioFormatDescriptionGetStreamBasicDescription(formatDescription)?.pointee
    else {
        return nil
    }

    let inputChannels = Int(asbd.mChannelsPerFrame)
    let inputSampleRate = asbd.mSampleRate
    let frameCount = CMSampleBufferGetNumSamples(sampleBuffer)
    if frameCount == 0 { return [] }

    // Get audio data into a buffer list
    var blockBuffer: CMBlockBuffer?
    var audioBufferList = AudioBufferList(
        mNumberBuffers: 1,
        mBuffers: AudioBuffer(mNumberChannels: 0, mDataByteSize: 0, mData: nil)
    )

    let status = CMSampleBufferGetAudioBufferListWithRetainedBlockBuffer(
        sampleBuffer,
        bufferListSizeNeededOut: nil,
        bufferListOut: &audioBufferList,
        bufferListSize: MemoryLayout<AudioBufferList>.size,
        blockBufferAllocator: nil,
        blockBufferMemoryAllocator: nil,
        flags: kCMSampleBufferFlag_AudioBufferList_Assure16ByteAlignment,
        blockBufferOut: &blockBuffer
    )

    guard status == noErr else {
        log("CMSampleBufferGetAudioBufferList failed: \(status)")
        return nil
    }

    let buffer = audioBufferList.mBuffers
    guard let mData = buffer.mData else { return nil }
    let byteSize = Int(buffer.mDataByteSize)

    // Determine input format and convert to Float32 mono
    let isFloat = (asbd.mFormatFlags & kAudioFormatFlagIsFloat) != 0
    let bitsPerSample = Int(asbd.mBitsPerChannel)

    var monoFloat: [Float] = []
    monoFloat.reserveCapacity(frameCount)

    if isFloat && bitsPerSample == 32 {
        let pointer = mData.bindMemory(to: Float.self, capacity: byteSize / 4)
        if inputChannels == 1 {
            for i in 0..<frameCount {
                monoFloat.append(pointer[i])
            }
        } else {
            // Interleaved or non-interleaved? Check format flag.
            let isInterleaved = (asbd.mFormatFlags & kAudioFormatFlagIsNonInterleaved) == 0
            if isInterleaved {
                for i in 0..<frameCount {
                    var sum: Float = 0
                    for c in 0..<inputChannels {
                        sum += pointer[i * inputChannels + c]
                    }
                    monoFloat.append(sum / Float(inputChannels))
                }
            } else {
                // Non-interleaved: only handle the first channel for now
                for i in 0..<frameCount {
                    monoFloat.append(pointer[i])
                }
            }
        }
    } else if !isFloat && bitsPerSample == 16 {
        let pointer = mData.bindMemory(to: Int16.self, capacity: byteSize / 2)
        let scale: Float = 1.0 / 32768.0
        if inputChannels == 1 {
            for i in 0..<frameCount {
                monoFloat.append(Float(pointer[i]) * scale)
            }
        } else {
            for i in 0..<frameCount {
                var sum: Float = 0
                for c in 0..<inputChannels {
                    sum += Float(pointer[i * inputChannels + c]) * scale
                }
                monoFloat.append(sum / Float(inputChannels))
            }
        }
    } else {
        log("unsupported audio format: bits=\(bitsPerSample) float=\(isFloat)")
        return nil
    }

    // Resample to target sample rate using linear interpolation
    if abs(inputSampleRate - targetSampleRate) < 1.0 {
        return monoFloat
    }
    return resample(monoFloat, from: inputSampleRate, to: targetSampleRate)
}

/// Simple linear interpolation resampler. Good enough for speech at 16kHz target.
func resample(_ input: [Float], from sourceRate: Double, to targetRate: Double) -> [Float] {
    if input.isEmpty { return [] }
    let ratio = sourceRate / targetRate
    let outputCount = Int(Double(input.count) / ratio)
    var output = [Float]()
    output.reserveCapacity(outputCount)
    for i in 0..<outputCount {
        let srcPos = Double(i) * ratio
        let lo = Int(srcPos)
        let hi = min(lo + 1, input.count - 1)
        let frac = Float(srcPos - Double(lo))
        let sample = input[lo] * (1.0 - frac) + input[hi] * frac
        output.append(sample)
    }
    return output
}

// MARK: - Microphone capture (AVAudioEngine)

final class MicCapture {
    private let engine = AVAudioEngine()
    private let buffer: AudioRingBuffer
    private let targetSampleRate: Double
    private var converter: AVAudioConverter?
    private var converterOutputFormat: AVAudioFormat?

    init(buffer: AudioRingBuffer, targetSampleRate: Int) {
        self.buffer = buffer
        self.targetSampleRate = Double(targetSampleRate)
    }

    func start() throws {
        let input = engine.inputNode
        let inputFormat = input.outputFormat(forBus: 0)

        guard
            let monoFormat = AVAudioFormat(
                commonFormat: .pcmFormatFloat32,
                sampleRate: targetSampleRate,
                channels: 1,
                interleaved: false
            )
        else {
            throw NSError(
                domain: "MicCapture", code: 1,
                userInfo: [NSLocalizedDescriptionKey: "failed to create target format"])
        }

        self.converterOutputFormat = monoFormat
        self.converter = AVAudioConverter(from: inputFormat, to: monoFormat)

        log(
            "mic input format: \(inputFormat.sampleRate) Hz, \(inputFormat.channelCount) ch -> 16k mono"
        )

        input.installTap(onBus: 0, bufferSize: 4096, format: inputFormat) {
            [weak self] (pcmBuffer, _) in
            guard let self = self else { return }
            self.handleInputBuffer(pcmBuffer)
        }

        try engine.start()
    }

    func stop() {
        engine.inputNode.removeTap(onBus: 0)
        engine.stop()
    }

    private func handleInputBuffer(_ inputBuffer: AVAudioPCMBuffer) {
        guard let converter = converter, let outputFormat = converterOutputFormat else { return }

        let outputCapacity = AVAudioFrameCount(
            Double(inputBuffer.frameLength) * outputFormat.sampleRate
                / inputBuffer.format.sampleRate + 32)
        guard
            let outputBuffer = AVAudioPCMBuffer(
                pcmFormat: outputFormat, frameCapacity: outputCapacity)
        else { return }

        var error: NSError?
        var supplied = false
        let status = converter.convert(to: outputBuffer, error: &error) { _, outStatus in
            if supplied {
                outStatus.pointee = .noDataNow
                return nil
            }
            supplied = true
            outStatus.pointee = .haveData
            return inputBuffer
        }

        if let err = error {
            log("mic converter error: \(err.localizedDescription)")
            return
        }
        if status == .error || status == .endOfStream {
            return
        }

        let frames = Int(outputBuffer.frameLength)
        guard frames > 0, let channelData = outputBuffer.floatChannelData else { return }
        let samples = Array(UnsafeBufferPointer(start: channelData[0], count: frames))
        buffer.append(samples)
    }
}

// MARK: - System audio capture (ScreenCaptureKit)

final class SystemAudioCapture: NSObject, SCStreamOutput, SCStreamDelegate {
    private var stream: SCStream?
    private let buffer: AudioRingBuffer
    private let targetSampleRate: Double
    private let outputQueue = DispatchQueue(label: "meeting-recorder.sc-output")

    init(buffer: AudioRingBuffer, targetSampleRate: Int) {
        self.buffer = buffer
        self.targetSampleRate = Double(targetSampleRate)
        super.init()
    }

    func start() async throws {
        let content = try await SCShareableContent.excludingDesktopWindows(
            false, onScreenWindowsOnly: true)
        guard let display = content.displays.first else {
            throw NSError(
                domain: "SystemAudio", code: 1,
                userInfo: [NSLocalizedDescriptionKey: "no displays available"])
        }

        let filter = SCContentFilter(
            display: display, excludingApplications: [], exceptingWindows: [])
        let config = SCStreamConfiguration()
        config.capturesAudio = true
        config.excludesCurrentProcessAudio = true
        config.sampleRate = Int(targetSampleRate)
        config.channelCount = 1
        // Minimal video config (required even when we only want audio)
        config.width = 2
        config.height = 2
        config.minimumFrameInterval = CMTime(value: 1, timescale: 1)  // 1 fps
        config.queueDepth = 8

        let stream = SCStream(filter: filter, configuration: config, delegate: self)
        try stream.addStreamOutput(self, type: .audio, sampleHandlerQueue: outputQueue)
        try await stream.startCapture()
        self.stream = stream
        log("system audio capture started")
    }

    func stop() async {
        guard let stream = stream else { return }
        do {
            try await stream.stopCapture()
        } catch {
            log("error stopping SC stream: \(error.localizedDescription)")
        }
        self.stream = nil
    }

    func stream(
        _ stream: SCStream, didOutputSampleBuffer sampleBuffer: CMSampleBuffer, of type: SCStreamOutputType
    ) {
        guard type == .audio else { return }
        guard CMSampleBufferIsValid(sampleBuffer) else { return }
        if let samples = convertSampleBufferToMonoFloat32(
            sampleBuffer, targetSampleRate: targetSampleRate)
        {
            buffer.append(samples)
        }
    }

    func stream(_ stream: SCStream, didStopWithError error: Error) {
        log("SC stream stopped with error: \(error.localizedDescription)")
    }
}

// MARK: - Permission checks

func hasMicrophonePermission() -> Bool {
    let status = AVCaptureDevice.authorizationStatus(for: .audio)
    return status == .authorized
}

func requestMicrophonePermission() async -> Bool {
    return await withCheckedContinuation { cont in
        AVCaptureDevice.requestAccess(for: .audio) { granted in
            cont.resume(returning: granted)
        }
    }
}

func hasScreenRecordingPermission() -> Bool {
    return CGPreflightScreenCaptureAccess()
}

// MARK: - Mixer + chunk writer

final class MeetingRecorder {
    private let micBuffer: AudioRingBuffer
    private let systemBuffer: AudioRingBuffer
    private let outputDir: String
    private let chunkSamples: Int
    private let sampleRate: Int
    private let chunkMs: Int

    private var chunkIndex: Int = 0
    private let startWallClock: Date
    private var chunkTimer: DispatchSourceTimer?
    private let queue = DispatchQueue(label: "meeting-recorder.chunker")

    init(outputDir: String, sampleRate: Int, chunkMs: Int) {
        self.outputDir = outputDir
        self.sampleRate = sampleRate
        self.chunkMs = chunkMs
        self.chunkSamples = chunkMs * sampleRate / 1000
        self.micBuffer = AudioRingBuffer(maxSeconds: 600, sampleRate: sampleRate)
        self.systemBuffer = AudioRingBuffer(maxSeconds: 600, sampleRate: sampleRate)
        self.startWallClock = Date()
    }

    func micRing() -> AudioRingBuffer { micBuffer }
    func systemRing() -> AudioRingBuffer { systemBuffer }

    func startChunkTimer() {
        let timer = DispatchSource.makeTimerSource(queue: queue)
        timer.schedule(
            deadline: .now() + .milliseconds(chunkMs),
            repeating: .milliseconds(chunkMs)
        )
        timer.setEventHandler { [weak self] in
            self?.writeChunk(final: false)
        }
        timer.resume()
        self.chunkTimer = timer
    }

    func stopAndFlush() {
        chunkTimer?.cancel()
        chunkTimer = nil
        // Write any remaining audio (may be < chunkSamples)
        writeChunk(final: true)
    }

    private func writeChunk(final: Bool) {
        let micSamples = micBuffer.drain(upTo: chunkSamples)
        let sysSamples = systemBuffer.drain(upTo: chunkSamples)

        let frames = max(micSamples.count, sysSamples.count)
        if frames == 0 { return }

        // Mix: pad shorter buffer with zeros, sum with attenuation, clip.
        var mixed = [Float](repeating: 0, count: frames)
        for i in 0..<frames {
            let m = i < micSamples.count ? micSamples[i] : 0
            let s = i < sysSamples.count ? sysSamples[i] : 0
            // Attenuate to prevent clipping when both are loud.
            let mixedSample = (m + s) * 0.7
            mixed[i] = max(-1.0, min(1.0, mixedSample))
        }

        let startedAtMs =
            Int(Date().timeIntervalSince(startWallClock) * 1000) - (frames * 1000 / sampleRate)
        let durationMs = frames * 1000 / sampleRate
        let filename = "mixed_\(chunkIndex)_\(startedAtMs).wav"
        let path = (outputDir as NSString).appendingPathComponent(filename)

        do {
            try writeWav(samples: mixed, sampleRate: sampleRate, to: path)
            emitChunk(
                stream: "mixed",
                wavPath: path,
                startedAtMs: max(0, startedAtMs),
                durationMs: durationMs,
                sampleRate: sampleRate
            )
            chunkIndex += 1
        } catch {
            emitError("failed to write chunk: \(error.localizedDescription)")
        }
    }
}

// MARK: - Stdin command listener

func startStdinCommandLoop(onStop: @escaping () -> Void) {
    let queue = DispatchQueue(label: "meeting-recorder.stdin")
    queue.async {
        while let line = readLine(strippingNewline: true) {
            if line.isEmpty { continue }
            guard let data = line.data(using: .utf8),
                let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                let command = json["command"] as? String
            else {
                log("invalid stdin line: \(line)")
                continue
            }
            switch command {
            case "stop":
                onStop()
                return
            default:
                log("unknown command: \(command)")
            }
        }
        // EOF on stdin → treat as stop request
        onStop()
    }
}

// MARK: - Main

@main
struct App {
    static func main() async {
        let args = Args.parse()
        if args.outputDir.isEmpty {
            emitError("--output-dir is required")
            exit(2)
        }

        // Ensure output dir exists
        do {
            try FileManager.default.createDirectory(
                atPath: args.outputDir, withIntermediateDirectories: true)
        } catch {
            emitError("failed to create output dir: \(error.localizedDescription)")
            exit(2)
        }

        // Permission checks
        if !hasMicrophonePermission() {
            log("microphone permission not granted, requesting...")
            let granted = await requestMicrophonePermission()
            if !granted {
                emitPermissionDenied(
                    kind: "microphone", message: "Microphone access is required for meetings")
                exit(3)
            }
        }
        if !hasScreenRecordingPermission() {
            emitPermissionDenied(
                kind: "screen_recording",
                message:
                    "Screen Recording permission is required to capture meeting audio. Open System Settings → Privacy & Security → Screen Recording and enable VoiceTypr."
            )
            exit(3)
        }

        let recorder = MeetingRecorder(
            outputDir: args.outputDir, sampleRate: args.sampleRate, chunkMs: args.chunkMs)

        let mic = MicCapture(buffer: recorder.micRing(), targetSampleRate: args.sampleRate)
        let system = SystemAudioCapture(
            buffer: recorder.systemRing(), targetSampleRate: args.sampleRate)

        do {
            try mic.start()
        } catch {
            emitError("mic capture failed: \(error.localizedDescription)")
            exit(4)
        }

        do {
            try await system.start()
        } catch {
            emitError("system audio capture failed: \(error.localizedDescription)")
            mic.stop()
            exit(4)
        }

        recorder.startChunkTimer()
        emitReady()
        log("meeting recorder ready")

        // Set up shutdown coordination
        let stopOnce = StopOnce()

        let stopBlock: () -> Void = { [recorder, mic, system] in
            Task {
                await stopOnce.runOnce {
                    log("stopping recorder...")
                    recorder.stopAndFlush()
                    mic.stop()
                    await system.stop()
                    emitStopped()
                    fflush(stdout)
                    exit(0)
                }
            }
        }

        // Trap signals for clean shutdown
        let sigHandler: @convention(c) (Int32) -> Void = { _ in
            // Signal handlers must be async-signal-safe; just write to stderr
            // and exit. The atexit-style cleanup happens via stdin EOF / SIGTERM.
            fputs("[meeting-recorder] received signal, exiting\n", stderr)
            _exit(0)
        }
        signal(SIGINT, sigHandler)
        signal(SIGTERM, sigHandler)
        signal(SIGPIPE, SIG_IGN)

        startStdinCommandLoop(onStop: stopBlock)

        // Keep main alive
        await withCheckedContinuation { (_: CheckedContinuation<Void, Never>) in
            // Never resume — process exits via stopBlock or signal
        }
    }
}

actor StopOnce {
    private var done = false
    func runOnce(_ block: @Sendable () async -> Void) async {
        if done { return }
        done = true
        await block()
    }
}
