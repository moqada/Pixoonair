use serde::{Deserialize, Serialize};
use tauri_plugin_store::Store;

const DEFAULT_GIF_FILE_ID: &str = "group1/M00/AD/F8/L1ghbmAJ7TmEfZ4QAAAAALoykrw2568328";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GifFileType {
    Id,
    Url,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub target_device_name: String,
    pub gif_file_id: String,
    pub gif_file_url: String,
    pub gif_file_type: GifFileType,
}

impl AppSettings {
    pub fn load_from_store<R: tauri::Runtime>(store: &Store<R>) -> Self {
        let target_device_name = store
            .get("appSettings.targetDeviceName")
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();

        let gif_file_id = store
            .get("appSettings.gifFileId")
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or(DEFAULT_GIF_FILE_ID.to_string());

        let gif_file_url = store
            .get("appSettings.gifFileUrl")
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();

        let gif_file_type = store
            .get("appSettings.gifFileType")
            .and_then(|v| v.as_str().map(String::from))
            .and_then(|s| match s.as_str() {
                "id" => Some(GifFileType::Id),
                "url" => Some(GifFileType::Url),
                _ => None,
            })
            .unwrap_or(GifFileType::Id);

        AppSettings {
            target_device_name,
            gif_file_id,
            gif_file_url,
            gif_file_type,
        }
    }

    pub fn save<R: tauri::Runtime>(&self, store: &Store<R>) -> Result<(), String> {
        store.set(
            "appSettings.targetDeviceName",
            self.target_device_name.clone(),
        );
        store.set("appSettings.gifFileId", self.gif_file_id.clone());
        store.set("appSettings.gifFileUrl", self.gif_file_url.clone());
        let gif_file_type_str = match self.gif_file_type {
            GifFileType::Id => "id",
            GifFileType::Url => "url",
        };
        store.set("appSettings.gifFileType", gif_file_type_str);
        Ok(())
    }
}
