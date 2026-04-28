import type {
  MeetingSegment,
  MeetingSegmentEvent,
  MeetingStatus,
  MeetingStatusEvent,
} from "@/types/meetings";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

interface LiveMeetingState {
  segments: MeetingSegment[];
  status: MeetingStatus | "idle";
  elapsedSeconds: number;
}

export function useLiveMeeting(meetingId: string | null): LiveMeetingState {
  const [segments, setSegments] = useState<MeetingSegment[]>([]);
  const [status, setStatus] = useState<MeetingStatus | "idle">("idle");
  const [elapsedSeconds, setElapsedSeconds] = useState(0);
  const startedRef = useRef<number | null>(null);

  useEffect(() => {
    if (!meetingId) {
      setSegments([]);
      setStatus("idle");
      setElapsedSeconds(0);
      startedRef.current = null;
      return;
    }

    setSegments([]);
    setStatus("starting");
    startedRef.current = Date.now();

    const unlistens: Array<() => void> = [];
    let cancelled = false;

    listen<MeetingSegmentEvent>("meeting:segment", (event) => {
      if (cancelled || event.payload.meeting_id !== meetingId) return;
      setSegments((prev) => [
        ...prev,
        {
          speaker: event.payload.speaker,
          text: event.payload.text,
          started_at_ms: event.payload.started_at_ms,
          duration_ms: event.payload.duration_ms,
        },
      ]);
    }).then((u) => unlistens.push(u));

    listen<MeetingStatusEvent>("meeting:status", (event) => {
      if (cancelled || event.payload.meeting_id !== meetingId) return;
      setStatus(event.payload.status);
    }).then((u) => unlistens.push(u));

    const tick = setInterval(() => {
      if (startedRef.current) {
        setElapsedSeconds(Math.floor((Date.now() - startedRef.current) / 1000));
      }
    }, 1000);

    return () => {
      cancelled = true;
      clearInterval(tick);
      unlistens.forEach((u) => u());
    };
  }, [meetingId]);

  return { segments, status, elapsedSeconds };
}
