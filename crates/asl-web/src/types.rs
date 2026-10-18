//! Web Dashboard and REST API Data Transfer Objects

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatusDto {
    pub callsign: String,
    pub local_node: String,
    pub connected_node: Option<String>,
    pub state: String,
    pub cos_active: bool,
    pub ptt_active: bool,
    pub rf_freq: String,
    pub rf_ctcss: f32,
    pub squelch_level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectRequest {
    pub target_node: String,
    pub host: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtmfRequest {
    pub digit: char,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
}
