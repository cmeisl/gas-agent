use axum::{
    extract::rejection::JsonRejection, http::StatusCode, response::IntoResponse, routing::post,
    Router,
};
use gas_agent::AgentPayload;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::{info, warn};

/// The payload structure sent by the gas agent
#[derive(Debug, Deserialize, Serialize)]
struct AgentSubmission {
    payload: AgentPayload,
    signature: String,
    network_signature: String,
}

async fn handle_agent_publish(
    payload: Result<axum::Json<AgentSubmission>, JsonRejection>,
) -> impl IntoResponse {
    match payload {
        Ok(axum::Json(submission)) => {
            info!("═══════════════════════════════════════════════════════════════");
            info!("RECEIVED AGENT SUBMISSION");
            info!("═══════════════════════════════════════════════════════════════");
            info!("System:         {:?}", submission.payload.system);
            info!("Network:        {:?}", submission.payload.network);
            info!("From Block:     {}", submission.payload.from_block);
            info!("Settlement:     {:?}", submission.payload.settlement);
            info!(
                "Price:          {} {:?}",
                submission.payload.price, submission.payload.unit
            );
            info!("Timestamp:      {}", submission.payload.timestamp);
            info!("Schema Version: {}", submission.payload.schema_version);
            info!("───────────────────────────────────────────────────────────────");
            info!(
                "Signature:         {}...",
                &submission.signature[..20.min(submission.signature.len())]
            );
            info!(
                "Network Signature: {}...",
                &submission.network_signature[..20.min(submission.network_signature.len())]
            );
            info!("═══════════════════════════════════════════════════════════════");
            (StatusCode::OK, "OK".to_string())
        }
        Err(rejection) => {
            warn!("═══════════════════════════════════════════════════════════════");
            warn!("INVALID PAYLOAD RECEIVED");
            warn!("═══════════════════════════════════════════════════════════════");
            warn!("Error: {}", rejection);
            if let JsonRejection::JsonDataError(ref err) = rejection {
                warn!("Details: {}", err.body_text());
            } else if let JsonRejection::JsonSyntaxError(ref err) = rejection {
                warn!("Details: {}", err.body_text());
            }
            warn!("═══════════════════════════════════════════════════════════════");
            (
                StatusCode::BAD_REQUEST,
                format!("Invalid payload: {}", rejection),
            )
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let app = Router::new()
        .route("/v0/agents", post(handle_agent_publish))
        .fallback(|req: axum::http::Request<axum::body::Body>| async move {
            warn!("Unhandled request: {} {}", req.method(), req.uri());
            (StatusCode::NOT_FOUND, "Not Found")
        });

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Mock Collector listening on http://{}", addr);
    info!("Expecting POST requests at http://{}/v0/agents", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
