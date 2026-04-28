use super::storage;
use super::types::{Meeting, MeetingSegment, Speaker};
use crate::whisper::cache::TranscriberCache;
use crate::whisper::manager::WhisperManager;
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::async_runtime::{Mutex as AsyncMutex, RwLock as AsyncRwLock};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;
use tokio::sync::mpsc;

/// Active meeting session state. Stored in `MeetingSessions` keyed by meeting_id.
pub struct ActiveMeeting {
    pub id: String,
    pub started_at: chrono::DateTime<Utc>,
    pub child: Option<CommandChild>,
    pub recordings_dir: PathBuf,
    pub segments: Vec<MeetingSegment>,
    pub language: Option<String>,
    pub title: String,
}

impl ActiveMeeting {
    pub fn snapshot(&self) -> Meeting {
        let duration = (Utc::now() - self.started_at).num_seconds().max(0) as u64;
        Meeting {
            id: self.id.clone(),
            title: self.title.clone(),
            started_at: self.started_at.to_rfc3339(),
            ended_at: None,
            duration_seconds: duration,
            language: self.language.clone(),
            segments: self.segments.clone(),
            summary: None,
        }
    }
}

pub type MeetingSessions = AsyncMutex<HashMap<String, Arc<AsyncMutex<ActiveMeeting>>>>;

pub fn new_sessions_state() -> MeetingSessions {
    AsyncMutex::new(HashMap::new())
}

/// Sidecar protocol events
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum SidecarEvent {
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "chunk")]
    Chunk {
        stream: String,
        wav_path: String,
        started_at_ms: u64,
        duration_ms: u64,
        #[allow(dead_code)]
        sample_rate: u32,
    },
    #[serde(rename = "permission_denied")]
    PermissionDenied { kind: String, message: String },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "level")]
    #[allow(dead_code)]
    Level {
        stream: String,
        rms: f32,
    },
}

#[derive(Clone)]
pub struct StartParams {
    pub title: String,
    pub language: Option<String>,
    pub model_path: Option<PathBuf>,
}

/// Starts the sidecar, spawns the chunk-handling task, returns the meeting id.
pub async fn start(app: AppHandle, params: StartParams) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let started_at = Utc::now();
    let recordings_dir = storage::recordings_dir(&app, &id)?;

    // Spawn sidecar
    let sidecar_cmd = app
        .shell()
        .sidecar("meeting-recorder")
        .map_err(|e| format!("failed to resolve sidecar: {}", e))?
        .args([
            "--output-dir",
            recordings_dir.to_string_lossy().as_ref(),
            "--chunk-ms",
            "15000",
            "--sample-rate",
            "16000",
        ]);

    let (mut rx, child) = sidecar_cmd
        .spawn()
        .map_err(|e| format!("failed to spawn sidecar: {}", e))?;

    let active = Arc::new(AsyncMutex::new(ActiveMeeting {
        id: id.clone(),
        started_at,
        child: Some(child),
        recordings_dir: recordings_dir.clone(),
        segments: vec![],
        language: params.language.clone(),
        title: params.title.clone(),
    }));

    // Register session
    let sessions: tauri::State<MeetingSessions> = app.state();
    sessions.lock().await.insert(id.clone(), active.clone());

    // Persist initial meeting record
    let meeting_init = active.lock().await.snapshot();
    let _ = storage::save_meeting(&app, &meeting_init);

    emit_status(&app, &id, "starting", None);

    // Spawn task that reads sidecar stdout
    let app_clone = app.clone();
    let id_clone = id.clone();
    let active_clone = active.clone();
    let model_path = params.model_path.clone();
    let language = params.language.clone();

    tauri::async_runtime::spawn(async move {
        let (segment_tx, mut segment_rx) = mpsc::unbounded_channel::<MeetingSegment>();

        // Spawn transcription worker (sequential so we don't oversubscribe whisper).
        let app_for_worker = app_clone.clone();
        let id_for_worker = id_clone.clone();
        let active_for_worker = active_clone.clone();
        let worker_model_path = model_path.clone();
        let worker_language = language.clone();

        let (chunk_tx, mut chunk_rx) =
            mpsc::unbounded_channel::<(String, String, u64, u64)>(); // (stream, path, started_at_ms, duration_ms)

        tauri::async_runtime::spawn(async move {
            while let Some((stream, wav_path, started_at_ms, duration_ms)) = chunk_rx.recv().await {
                let speaker = match stream.as_str() {
                    "mic" => Speaker::Me,
                    "system" => Speaker::Them,
                    _ => Speaker::Mixed,
                };
                let text = match transcribe_wav(
                    &app_for_worker,
                    &wav_path,
                    worker_model_path.as_deref(),
                    worker_language.as_deref(),
                )
                .await
                {
                    Ok(t) => t,
                    Err(e) => {
                        log::warn!("transcribe failed for {}: {}", wav_path, e);
                        // Still emit the chunk so user knows progress; just empty text.
                        String::new()
                    }
                };
                // Cleanup wav file on disk
                let _ = std::fs::remove_file(&wav_path);

                let trimmed = text.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let segment = MeetingSegment {
                    speaker,
                    text: trimmed.to_string(),
                    started_at_ms,
                    duration_ms,
                };
                // Append to active meeting + persist + emit
                {
                    let mut guard = active_for_worker.lock().await;
                    guard.segments.push(segment.clone());
                    let snapshot = guard.snapshot();
                    drop(guard);
                    let _ = storage::save_meeting(&app_for_worker, &snapshot);
                }
                let _ = segment_tx.send(segment.clone());
                let _ = app_for_worker.emit(
                    "meeting:segment",
                    json!({
                        "meeting_id": id_for_worker,
                        "speaker": match speaker {
                            Speaker::Me => "me",
                            Speaker::Them => "them",
                            Speaker::Mixed => "mixed",
                        },
                        "text": segment.text,
                        "started_at_ms": segment.started_at_ms,
                        "duration_ms": segment.duration_ms,
                    }),
                );
            }
        });

        // Drain the segment_rx so it doesn't get dropped immediately. Not used downstream
        // — events drive the UI — but keeping the channel alive lets the worker send freely.
        tauri::async_runtime::spawn(async move { while segment_rx.recv().await.is_some() {} });

        // Read sidecar events
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes).to_string();
                    for piece in line.lines() {
                        let piece = piece.trim();
                        if piece.is_empty() {
                            continue;
                        }
                        match serde_json::from_str::<SidecarEvent>(piece) {
                            Ok(SidecarEvent::Ready) => {
                                emit_status(&app_clone, &id_clone, "recording", None);
                            }
                            Ok(SidecarEvent::Chunk {
                                stream,
                                wav_path,
                                started_at_ms,
                                duration_ms,
                                ..
                            }) => {
                                let _ = chunk_tx.send((stream, wav_path, started_at_ms, duration_ms));
                            }
                            Ok(SidecarEvent::PermissionDenied { kind, message }) => {
                                emit_status(&app_clone, &id_clone, "error", Some(&message));
                                let _ = app_clone.emit(
                                    "meeting:error",
                                    json!({
                                        "meeting_id": id_clone,
                                        "kind": kind,
                                        "message": message,
                                    }),
                                );
                            }
                            Ok(SidecarEvent::Error { message }) => {
                                let _ = app_clone.emit(
                                    "meeting:error",
                                    json!({
                                        "meeting_id": id_clone,
                                        "message": message,
                                    }),
                                );
                            }
                            Ok(SidecarEvent::Stopped) => {
                                log::info!("sidecar reports stopped for {}", id_clone);
                            }
                            Ok(SidecarEvent::Level { .. }) => {
                                // ignore for now
                            }
                            Err(e) => {
                                log::warn!("sidecar JSON parse error: {} (line: {})", e, piece);
                            }
                        }
                    }
                }
                CommandEvent::Stderr(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    for l in line.lines() {
                        if !l.trim().is_empty() {
                            log::debug!("[meeting-sidecar] {}", l);
                        }
                    }
                }
                CommandEvent::Error(e) => {
                    log::error!("sidecar process error: {}", e);
                    let _ = app_clone.emit(
                        "meeting:error",
                        json!({
                            "meeting_id": id_clone,
                            "message": format!("sidecar error: {}", e),
                        }),
                    );
                }
                CommandEvent::Terminated(_) => {
                    log::info!("sidecar terminated for {}", id_clone);
                    break;
                }
                _ => {}
            }
        }
        drop(chunk_tx);
    });

    Ok(id)
}

/// Send stop command to the sidecar, await termination, finalize meeting record.
pub async fn stop(app: AppHandle, meeting_id: String) -> Result<Meeting, String> {
    let sessions: tauri::State<MeetingSessions> = app.state();
    let active = {
        let map = sessions.lock().await;
        map.get(&meeting_id).cloned()
    };
    let Some(active) = active else {
        return Err("meeting not found".to_string());
    };

    emit_status(&app, &meeting_id, "stopping", None);

    // Send stop command + close stdin
    {
        let mut guard = active.lock().await;
        if let Some(child) = guard.child.as_mut() {
            let cmd = b"{\"command\":\"stop\"}\n".to_vec();
            if let Err(e) = child.write(&cmd) {
                log::warn!("failed to send stop to sidecar: {}", e);
            }
        }
    }

    // Give the sidecar up to 30s to flush remaining chunks and exit cleanly.
    let mut elapsed_ms = 0u64;
    let deadline_ms = 30_000u64;
    while elapsed_ms < deadline_ms {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        elapsed_ms += 500;
        let guard = active.lock().await;
        if guard.child.is_none() {
            break;
        }
        // check if terminated: we cannot poll directly, but the reader task
        // will set child to None on Terminated. Keep waiting otherwise.
        drop(guard);
    }

    // Force kill if still around
    {
        let mut guard = active.lock().await;
        if let Some(child) = guard.child.take() {
            let _ = child.kill();
        }
    }

    // Finalize meeting record
    let final_meeting = {
        let guard = active.lock().await;
        let mut m = guard.snapshot();
        m.ended_at = Some(Utc::now().to_rfc3339());
        m.duration_seconds = (Utc::now() - guard.started_at).num_seconds().max(0) as u64;
        m
    };
    storage::save_meeting(&app, &final_meeting)?;

    // Remove from active sessions
    sessions.lock().await.remove(&meeting_id);

    emit_status(&app, &meeting_id, "complete", None);
    Ok(final_meeting)
}

pub async fn list(app: AppHandle) -> Result<Vec<super::types::MeetingIndexEntry>, String> {
    storage::load_index(&app)
}

pub async fn get(app: AppHandle, id: String) -> Result<Meeting, String> {
    storage::load_meeting(&app, &id)
}

pub async fn remove(app: AppHandle, id: String) -> Result<(), String> {
    // If active, stop first
    let sessions: tauri::State<MeetingSessions> = app.state();
    let exists = sessions.lock().await.contains_key(&id);
    if exists {
        let _ = stop(app.clone(), id.clone()).await;
    }
    storage::delete_meeting(&app, &id)
}

fn emit_status(app: &AppHandle, id: &str, status: &str, message: Option<&str>) {
    let _ = app.emit(
        "meeting:status",
        json!({
            "meeting_id": id,
            "status": status,
            "message": message,
        }),
    );
}

/// Transcribe a WAV file using the existing whisper pipeline.
/// Resolves the model path lazily using the WhisperManager state.
async fn transcribe_wav(
    app: &AppHandle,
    wav_path: &str,
    explicit_model_path: Option<&std::path::Path>,
    language: Option<&str>,
) -> Result<String, String> {
    let model_path: PathBuf = if let Some(p) = explicit_model_path {
        p.to_path_buf()
    } else {
        let whisper_state = app.state::<AsyncRwLock<WhisperManager>>();
        let manager = whisper_state.read().await;
        let current_model = manager
            .get_downloaded_model_names()
            .into_iter()
            .next()
            .ok_or_else(|| "no whisper model available".to_string())?;
        manager
            .get_model_path(&current_model)
            .ok_or_else(|| format!("model path missing for {}", current_model))?
    };

    let cache_state = app.state::<AsyncMutex<TranscriberCache>>();
    let transcriber = {
        let mut cache = cache_state.lock().await;
        cache.get_or_create(&model_path)?
    };

    let path = std::path::Path::new(wav_path).to_path_buf();
    let lang = language.map(|s| s.to_string());

    tokio::task::spawn_blocking(move || {
        transcriber.transcribe_with_translation(&path, lang.as_deref(), false)
    })
    .await
    .map_err(|e| format!("join error: {}", e))?
}
