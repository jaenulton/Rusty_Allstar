//! # asl-web
//!
//! Embedded Web Dashboard and REST API serialization definitions for Rusty_Allstar.
//! Includes embedded HTML/CSS/JS frontend assets for micro-nodes and desktop servers.

pub mod types;

pub use types::{ApiResponse, ConnectRequest, DtmfRequest, NodeStatusDto};

/// Embedded single-page web dashboard HTML, CSS, and JS
pub const INDEX_HTML: &str = include_str!("../static/index.html");

/// Content-Type for HTML
pub const MIME_HTML: &str = "text/html; charset=utf-8";

/// Content-Type for JSON
pub const MIME_JSON: &str = "application/json";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_assets() {
        assert!(!INDEX_HTML.is_empty());
        assert!(INDEX_HTML.contains("Rusty_Allstar Control Panel"));
        assert!(INDEX_HTML.contains("api/status"));
    }

    #[test]
    fn test_dto_serialization() {
        let status = NodeStatusDto {
            callsign: "W1AW".to_string(),
            local_node: "1999".to_string(),
            connected_node: Some("50000".to_string()),
            state: "LINKED".to_string(),
            cos_active: true,
            ptt_active: false,
            rf_freq: "446.0000".to_string(),
            rf_ctcss: 100.0,
            squelch_level: 4,
        };

        let json = serde_json::to_string(&status).expect("serialization failed");
        assert!(json.contains("\"callsign\":\"W1AW\""));
        assert!(json.contains("\"cos_active\":true"));

        let deserialized: NodeStatusDto =
            serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(deserialized.local_node, "1999");
    }

    #[test]
    fn test_api_response() {
        let resp = ApiResponse {
            success: true,
            message: "Connected to 50000".to_string(),
        };
        let json = serde_json::to_string(&resp).expect("serialization failed");
        assert!(json.contains("\"success\":true"));
    }
}
