export type Speaker = "me" | "them" | "mixed";

export type MeetingStatus =
  | "starting"
  | "recording"
  | "stopping"
  | "transcribing"
  | "summarizing"
  | "complete"
  | "error";

export interface MeetingSegment {
  speaker: Speaker;
  text: string;
  started_at_ms: number;
  duration_ms: number;
}

export interface MeetingSummary {
  key_points: string[];
  action_items: string[];
  decisions: string[];
  raw: string;
}

export interface Meeting {
  id: string;
  title: string;
  started_at: string;
  ended_at?: string;
  duration_seconds: number;
  language?: string;
  segments: MeetingSegment[];
  summary: MeetingSummary | null;
}

export interface MeetingIndexEntry {
  id: string;
  title: string;
  started_at: string;
  duration_seconds: number;
  has_summary: boolean;
}

export interface MeetingStatusEvent {
  meeting_id: string;
  status: MeetingStatus;
  message?: string | null;
}

export interface MeetingSegmentEvent extends MeetingSegment {
  meeting_id: string;
}

export interface MeetingErrorEvent {
  meeting_id: string;
  kind?: string;
  message: string;
}
