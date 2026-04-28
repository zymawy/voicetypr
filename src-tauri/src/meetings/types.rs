use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Speaker {
    Me,
    Them,
    Mixed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MeetingSegment {
    pub speaker: Speaker,
    pub text: String,
    pub started_at_ms: u64,
    pub duration_ms: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MeetingSummary {
    #[serde(default)]
    pub key_points: Vec<String>,
    #[serde(default)]
    pub action_items: Vec<String>,
    #[serde(default)]
    pub decisions: Vec<String>,
    pub raw: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Meeting {
    pub id: String,
    pub title: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_seconds: u64,
    pub language: Option<String>,
    pub segments: Vec<MeetingSegment>,
    pub summary: Option<MeetingSummary>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MeetingIndexEntry {
    pub id: String,
    pub title: String,
    pub started_at: String,
    pub duration_seconds: u64,
    pub has_summary: bool,
}
