//! Web Server and REST API integration for `asl-node-cli` using Axum.

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use tower_http::cors::CorsLayer;
use tracing::info;

use asl_web::{
    types::{ApiResponse, ConnectRequest, DtmfRequest, NodeStatusDto},
    INDEX_HTML, MIME_HTML,
};

/// Shared state across the web server and the CLI / IAX2 node runtime
#[derive(Debug, Clone)]
pub struct AppState {
    pub callsign: Arc<RwLock<String>>,
    pub local_node: Arc<RwLock<String>>,
    pub connected_node: Arc<RwLock<Option<String>>>,
    pub link_state: Arc<RwLock<String>>,
    pub cos_active: Arc<RwLock<bool>>,
    pub ptt_active: Arc<RwLock<bool>>,
    pub rf_freq: Arc<RwLock<String>>,
    pub rf_ctcss: Arc<RwLock<f32>>,
    pub squelch_level: Arc<RwLock<u8>>,
    /// Channel sender to send control commands back to the IAX2 node runtime
    pub command_tx: tokio::sync::mpsc::Sender<NodeCommand>,
}

#[derive(Debug)]
pub enum NodeCommand {
    Connect { target_node: String, host: Option<String> },
    Disconnect,
    SendDtmf(char),
    ForceCwId,
}

pub async fn start_web_server(addr: SocketAddr, state: AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new()
        .route("/", get(serve_dashboard))
        .route("/api/status", get(get_status))
        .route("/api/connect", post(post_connect))
        .route("/api/disconnect", post(post_disconnect))
        .route("/api/dtmf", post(post_dtmf))
        .route("/api/cwid", post(post_cwid))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Embedded Web Dashboard & REST API listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn serve_dashboard() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, MIME_HTML)], Html(INDEX_HTML))
}

async fn get_status(State(state): State<AppState>) -> Json<NodeStatusDto> {
    let callsign = state.callsign.read().await.clone();
    let local_node = state.local_node.read().await.clone();
    let connected_node = state.connected_node.read().await.clone();
    let state_str = state.link_state.read().await.clone();
    let cos_active = *state.cos_active.read().await;
    let ptt_active = *state.ptt_active.read().await;
    let rf_freq = state.rf_freq.read().await.clone();
    let rf_ctcss = *state.rf_ctcss.read().await;
    let squelch_level = *state.squelch_level.read().await;

    Json(NodeStatusDto {
        callsign,
        local_node,
        connected_node,
        state: state_str,
        cos_active,
        ptt_active,
        rf_freq,
        rf_ctcss,
        squelch_level,
    })
}

async fn post_connect(
    State(state): State<AppState>,
    Json(payload): Json<ConnectRequest>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    let target = payload.target_node.clone();
    if state
        .command_tx
        .send(NodeCommand::Connect {
            target_node: payload.target_node,
            host: payload.host,
        })
        .await
        .is_err()
    {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                message: "Internal command channel closed".to_string(),
            }),
        ));
    }

    Ok(Json(ApiResponse {
        success: true,
        message: format!("Connect request sent for node {}", target),
    }))
}

async fn post_disconnect(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    if state.command_tx.send(NodeCommand::Disconnect).await.is_err() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                message: "Internal command channel closed".to_string(),
            }),
        ));
    }

    Ok(Json(ApiResponse {
        success: true,
        message: "Disconnect command dispatched".to_string(),
    }))
}

async fn post_dtmf(
    State(state): State<AppState>,
    Json(payload): Json<DtmfRequest>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    let digit = payload.digit;
    if state.command_tx.send(NodeCommand::SendDtmf(digit)).await.is_err() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                message: "Internal command channel closed".to_string(),
            }),
        ));
    }

    Ok(Json(ApiResponse {
        success: true,
        message: format!("DTMF '{}' transmitted", digit),
    }))
}

async fn post_cwid(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    if state.command_tx.send(NodeCommand::ForceCwId).await.is_err() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                message: "Internal command channel closed".to_string(),
            }),
        ));
    }

    Ok(Json(ApiResponse {
        success: true,
        message: "CW ID triggered".to_string(),
    }))
}
