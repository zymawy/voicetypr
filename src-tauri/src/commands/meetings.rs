use crate::meetings::session::{self, StartParams};
use crate::meetings::storage;
use crate::meetings::summary as summary_mod;
use crate::meetings::types::{Meeting, MeetingIndexEntry, MeetingSummary};
use chrono::Local;

#[tauri::command]
pub async fn start_meeting(
    app: tauri::AppHandle,
    title: Option<String>,
) -> Result<String, String> {
    let title = title
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| Local::now().format("Meeting on %b %d, %H:%M").to_string());

    let language = {
        use tauri_plugin_store::StoreExt;
        let store = app.store("settings").ok();
        store.and_then(|s| {
            s.get("language")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        })
    };

    session::start(
        app.clone(),
        StartParams {
            title,
            language,
            model_path: None,
        },
    )
    .await
}

#[tauri::command]
pub async fn stop_meeting(app: tauri::AppHandle, meeting_id: String) -> Result<Meeting, String> {
    session::stop(app, meeting_id).await
}

#[tauri::command]
pub async fn summarize_meeting(
    app: tauri::AppHandle,
    meeting_id: String,
) -> Result<MeetingSummary, String> {
    let mut meeting = storage::load_meeting(&app, &meeting_id)?;
    let summary = summary_mod::generate_summary(&app, &meeting).await?;
    meeting.summary = Some(summary.clone());
    storage::save_meeting(&app, &meeting)?;
    Ok(summary)
}

#[tauri::command]
pub async fn list_meetings(app: tauri::AppHandle) -> Result<Vec<MeetingIndexEntry>, String> {
    session::list(app).await
}

#[tauri::command]
pub async fn get_meeting(app: tauri::AppHandle, meeting_id: String) -> Result<Meeting, String> {
    session::get(app, meeting_id).await
}

#[tauri::command]
pub async fn delete_meeting(app: tauri::AppHandle, meeting_id: String) -> Result<(), String> {
    session::remove(app, meeting_id).await
}

#[tauri::command]
pub async fn rename_meeting(
    app: tauri::AppHandle,
    meeting_id: String,
    title: String,
) -> Result<Meeting, String> {
    let mut meeting = storage::load_meeting(&app, &meeting_id)?;
    meeting.title = title;
    storage::save_meeting(&app, &meeting)?;
    Ok(meeting)
}
