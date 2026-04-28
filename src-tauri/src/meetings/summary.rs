use super::types::{Meeting, MeetingSegment, MeetingSummary, Speaker};
use crate::commands::ai::{build_ai_provider_config_inner, call_ai_with_raw_prompt};

const SUMMARY_SYSTEM_PROMPT: &str = "You are a meticulous meeting note-taker. \
Read the transcript and produce ONLY a single valid JSON object — no preface, \
no fences, no commentary — that strictly conforms to the schema below.\n\n\
Schema (all keys required):\n\
{\n\
  \"key_points\": string[]   // 3-7 concise bullets covering main topics\n\
  \"action_items\": string[] // tasks; include the owner if mentioned\n\
  \"decisions\": string[]    // concrete decisions reached, if any\n\
  \"raw\": string             // full markdown summary; sections in order: ## Overview, ## Key Points, ## Action Items, ## Decisions\n\
}\n\n\
Rules:\n\
- Use empty arrays [] when a category has no entries.\n\
- Be specific. Quote concrete commitments verbatim.\n\
- Do NOT invent attendees, deadlines, or facts not in the transcript.\n\
- Preserve speaker labels (Me / Them) when relevant.";

pub fn build_summary_prompt(meeting: &Meeting) -> String {
    let mut transcript = String::new();
    for seg in &meeting.segments {
        let speaker = match seg.speaker {
            Speaker::Me => "Me",
            Speaker::Them => "Them",
            Speaker::Mixed => "Speaker",
        };
        transcript.push_str(&format!("{}: {}\n", speaker, seg.text.trim()));
    }
    if transcript.trim().is_empty() {
        transcript.push_str("(no transcript captured)");
    }
    format!(
        "{}\n\nTranscript:\n{}",
        SUMMARY_SYSTEM_PROMPT,
        transcript.trim()
    )
}

pub async fn generate_summary(
    app: &tauri::AppHandle,
    meeting: &Meeting,
) -> Result<MeetingSummary, String> {
    let (config, _model) = build_ai_provider_config_inner(app, false).await?;
    let prompt = build_summary_prompt(meeting);
    let response = call_ai_with_raw_prompt(&config, &prompt).await?;
    Ok(parse_summary_response(&response))
}

/// Best-effort JSON parsing. Falls back to `MeetingSummary { raw: response, ..Default }`
/// when parsing fails so we never lose the AI output.
pub fn parse_summary_response(response: &str) -> MeetingSummary {
    let cleaned = strip_code_fences(response);
    if let Ok(parsed) = serde_json::from_str::<MeetingSummary>(cleaned) {
        // If raw is empty, build one from the structured fields.
        if parsed.raw.trim().is_empty() {
            let raw = build_raw_markdown(&parsed);
            return MeetingSummary { raw, ..parsed };
        }
        return parsed;
    }
    MeetingSummary {
        raw: response.trim().to_string(),
        ..Default::default()
    }
}

fn strip_code_fences(s: &str) -> &str {
    let trimmed = s.trim();
    if let Some(rest) = trimmed.strip_prefix("```json") {
        let rest = rest.trim_start_matches('\n');
        if let Some(end) = rest.rfind("```") {
            return rest[..end].trim();
        }
    }
    if let Some(rest) = trimmed.strip_prefix("```") {
        let rest = rest.trim_start_matches('\n');
        if let Some(end) = rest.rfind("```") {
            return rest[..end].trim();
        }
    }
    trimmed
}

fn build_raw_markdown(s: &MeetingSummary) -> String {
    let mut md = String::from("## Overview\n\n");
    md.push_str("(generated)\n\n");

    md.push_str("## Key Points\n\n");
    for kp in &s.key_points {
        md.push_str(&format!("- {}\n", kp));
    }
    if s.key_points.is_empty() {
        md.push_str("- _none_\n");
    }
    md.push('\n');

    md.push_str("## Action Items\n\n");
    for ai in &s.action_items {
        md.push_str(&format!("- {}\n", ai));
    }
    if s.action_items.is_empty() {
        md.push_str("- _none_\n");
    }
    md.push('\n');

    md.push_str("## Decisions\n\n");
    for d in &s.decisions {
        md.push_str(&format!("- {}\n", d));
    }
    if s.decisions.is_empty() {
        md.push_str("- _none_\n");
    }
    md.push('\n');

    md
}

pub fn make_segment(text: String, speaker: Speaker, started_at_ms: u64, duration_ms: u64) -> MeetingSegment {
    MeetingSegment {
        speaker,
        text,
        started_at_ms,
        duration_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clean_json() {
        let resp = "{\"key_points\":[\"Discussed Q3 plan\"],\
            \"action_items\":[\"Hamza to send agenda\"],\
            \"decisions\":[\"Ship in May\"],\
            \"raw\":\"## Overview\\n...\"}";
        let s = parse_summary_response(resp);
        assert_eq!(s.key_points.len(), 1);
        assert_eq!(s.action_items.len(), 1);
        assert!(s.raw.contains("Overview"));
    }

    #[test]
    fn parses_fenced_json() {
        let resp = "```json\n{\"key_points\":[\"a\"],\"action_items\":[],\"decisions\":[],\"raw\":\"r\"}\n```";
        let s = parse_summary_response(resp);
        assert_eq!(s.key_points, vec!["a"]);
        assert_eq!(s.raw, "r");
    }

    #[test]
    fn falls_back_on_invalid_json() {
        let s = parse_summary_response("Sorry, I can't do that");
        assert!(s.raw.contains("Sorry"));
        assert!(s.key_points.is_empty());
    }

    #[test]
    fn builds_raw_when_empty() {
        let resp = r#"{"key_points":["a"],"action_items":[],"decisions":[],"raw":""}"#;
        let s = parse_summary_response(resp);
        assert!(s.raw.contains("Key Points"));
    }
}
