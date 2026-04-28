use crate::device_id;

/// Returns the stable, privacy-preserving device identifier used for licensing
#[tauri::command]
pub async fn get_device_id() -> Result<String, String> {
    device_id::get_device_hash()
}
