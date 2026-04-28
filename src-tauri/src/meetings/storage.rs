use super::types::{Meeting, MeetingIndexEntry};
use std::path::{Path, PathBuf};
use tauri::Manager;

const INDEX_FILE: &str = "index.json";

pub fn meetings_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to get app data dir: {}", e))?;
    let dir = base.join("meetings");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("failed to create meetings dir: {}", e))?;
    Ok(dir)
}

pub fn recordings_dir(app: &tauri::AppHandle, meeting_id: &str) -> Result<PathBuf, String> {
    let dir = meetings_dir(app)?.join("recordings").join(meeting_id);
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("failed to create recordings dir: {}", e))?;
    Ok(dir)
}

fn meeting_path(app: &tauri::AppHandle, id: &str) -> Result<PathBuf, String> {
    Ok(meetings_dir(app)?.join(format!("{}.json", id)))
}

fn index_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(meetings_dir(app)?.join(INDEX_FILE))
}

pub fn save_meeting(app: &tauri::AppHandle, meeting: &Meeting) -> Result<(), String> {
    let path = meeting_path(app, &meeting.id)?;
    write_atomic(&path, &serde_json::to_vec_pretty(meeting).map_err(|e| e.to_string())?)?;
    update_index_entry(app, meeting)?;
    Ok(())
}

pub fn load_meeting(app: &tauri::AppHandle, id: &str) -> Result<Meeting, String> {
    let path = meeting_path(app, id)?;
    let bytes = std::fs::read(&path)
        .map_err(|e| format!("failed to read meeting {}: {}", id, e))?;
    serde_json::from_slice::<Meeting>(&bytes).map_err(|e| format!("failed to parse meeting: {}", e))
}

pub fn delete_meeting(app: &tauri::AppHandle, id: &str) -> Result<(), String> {
    let path = meeting_path(app, id)?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("delete failed: {}", e))?;
    }

    // Also clean up any recordings dir
    let rec_dir = meetings_dir(app)?.join("recordings").join(id);
    if rec_dir.exists() {
        let _ = std::fs::remove_dir_all(&rec_dir);
    }

    // Update index
    let mut entries = load_index(app)?;
    entries.retain(|e| e.id != id);
    save_index(app, &entries)?;
    Ok(())
}

pub fn load_index(app: &tauri::AppHandle) -> Result<Vec<MeetingIndexEntry>, String> {
    let path = index_path(app)?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("failed to read index: {}", e))?;
    serde_json::from_slice::<Vec<MeetingIndexEntry>>(&bytes)
        .map_err(|e| format!("failed to parse index: {}", e))
}

fn save_index(app: &tauri::AppHandle, entries: &[MeetingIndexEntry]) -> Result<(), String> {
    let path = index_path(app)?;
    let bytes = serde_json::to_vec_pretty(entries).map_err(|e| e.to_string())?;
    write_atomic(&path, &bytes)
}

fn update_index_entry(app: &tauri::AppHandle, meeting: &Meeting) -> Result<(), String> {
    let mut entries = load_index(app)?;
    let entry = MeetingIndexEntry {
        id: meeting.id.clone(),
        title: meeting.title.clone(),
        started_at: meeting.started_at.clone(),
        duration_seconds: meeting.duration_seconds,
        has_summary: meeting.summary.is_some(),
    };
    if let Some(existing) = entries.iter_mut().find(|e| e.id == meeting.id) {
        *existing = entry;
    } else {
        entries.insert(0, entry);
    }
    save_index(app, &entries)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let tmp_path = path.with_extension("json.tmp");
    std::fs::write(&tmp_path, bytes).map_err(|e| format!("write failed: {}", e))?;
    std::fs::rename(&tmp_path, path).map_err(|e| format!("rename failed: {}", e))?;
    Ok(())
}
