use reqwest;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Device {
    pub device_name: String,
    pub device_id: u64,
    pub device_private_i_p: String,
    pub device_mac: String,
    pub hardware: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DeviceListResponse {
    return_code: u8,
    return_message: String,
    device_list: Vec<Device>,
}

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum ChannelId {
    Faces = 0,
    CloudChannel = 1,
    Visualizer = 2,
    Custom = 3,
    BlackScreen = 4,
    Unknown, // 未知の値が来た場合
}

impl From<u8> for ChannelId {
    fn from(value: u8) -> Self {
        match value {
            0 => ChannelId::Faces,
            1 => ChannelId::CloudChannel,
            2 => ChannelId::Visualizer,
            3 => ChannelId::Custom,
            4 => ChannelId::BlackScreen,
            _ => ChannelId::Unknown,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SelectIndexResponse {
    select_index: u8,
}

#[derive(Deserialize, Debug)]
struct ErrorCodeResponse {
    error_code: u8,
}

pub async fn get_devices() -> Result<Vec<Device>, String> {
    log::debug!("call get_devices");
    let client = reqwest::Client::new();
    let res = client
        .post("https://app.divoom-gz.com/Device/ReturnSameLANDevice")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let device_list_response = res
            .json::<DeviceListResponse>()
            .await
            .map_err(|e| e.to_string())?;
        if device_list_response.return_code == 0 {
            Ok(device_list_response.device_list)
        } else {
            Err(format!(
                "Error: {} (Code: {})",
                device_list_response.return_message, device_list_response.return_code
            ))
        }
    } else {
        Err(format!("HTTP error: {}", res.status()))
    }
}

pub async fn play_gif(ip: &str, file_name: &str) -> Result<(), String> {
    log::debug!("call play_gif");
    let client = reqwest::Client::new();
    let res = client
        .post(format!("http://{}/post", ip))
        .json(&serde_json::json!({
            "Command": "Device/PlayTFGif",
            "FileType": 2,
            "FileName": file_name,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let err_code_only_res = res
            .json::<ErrorCodeResponse>()
            .await
            .map_err(|e| e.to_string())?;
        if err_code_only_res.error_code == 0 {
            Ok(())
        } else {
            Err(format!("ErrorCode: {})", err_code_only_res.error_code))
        }
    } else {
        Err(format!("HTTP error: {}", res.status()))
    }
}

pub async fn play_divoom_gif(ip: &str, file_id: String) -> Result<(), String> {
    log::debug!("call play_divoom_gif");
    let client = reqwest::Client::new();
    let res = client
        .post(format!("http://{}/post", ip))
        .json(&serde_json::json!({
            "Command": "Draw/SendRemote",
            "FileId": file_id,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let err_code_res = res
            .json::<ErrorCodeResponse>()
            .await
            .map_err(|e| e.to_string())?;
        if err_code_res.error_code == 0 {
            Ok(())
        } else {
            Err(format!("ErrorCode: {})", err_code_res.error_code))
        }
    } else {
        Err(format!("HTTP error: {}", res.status()))
    }
}

pub async fn get_current_channel(ip: &str) -> Result<ChannelId, String> {
    log::debug!("call get_current_channel");
    let client = reqwest::Client::new();
    let res = client
        .post(format!("http://{}/post", ip))
        .json(&serde_json::json!({
            "Command": "Channel/GetIndex",
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let select_index_res = res
            .json::<SelectIndexResponse>()
            .await
            .map_err(|e| e.to_string())?;
        Ok(ChannelId::from(select_index_res.select_index))
    } else {
        Err(format!("HTTP error: {}", res.status()))
    }
}

pub async fn set_current_channel(ip: &str, select_index: ChannelId) -> Result<(), String> {
    log::debug!("call set_current_channel");
    let client = reqwest::Client::new();
    let res = client
        .post(format!("http://{}/post", ip))
        .json(&serde_json::json!({
            "Command": "Channel/SetIndex",
            "SelectIndex": select_index as u8,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let err_code_res = res
            .json::<ErrorCodeResponse>()
            .await
            .map_err(|e| e.to_string())?;
        if err_code_res.error_code == 0 {
            Ok(())
        } else {
            Err(format!("ErrorCode: {})", err_code_res.error_code))
        }
    } else {
        Err(format!("HTTP error: {}", res.status()))
    }
}
