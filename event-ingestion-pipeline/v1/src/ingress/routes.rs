use std::{sync::Arc, time::Duration};

use axum::{
    error_handling::HandleErrorLayer,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Semaphore;
use tower::ServiceBuilder;
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    config::Settings,
    domain::event::EventEnvelope,
    kafka::producer::Publisher,
    observability::{
        health::{HealthState, Liveness},
        metrics,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub health: Arc<HealthState>,
    pub publisher: Publisher,
    pub in_flight: Arc<Semaphore>, // bounds concurrent work to prevent overload
}

#[derive(Debug, Deserialize)]
pub struct IngestRequest {
    pub event_type: String,
    pub user_id: String,
    pub room_id: String,
    pub payload: Value,
    pub producer_timestamp: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct IngestAccepted {
    pub ok: bool,
    pub event_id: String,
    pub request_id: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub ok: bool,
    pub code: &'static str,
    pub message: String,
    pub request_id: String, // required for end-to-end traceability
}

/// Router enforces bounded input and execution time.
pub fn router(state: AppState) -> Router {
    let request_timeout = Duration::from_millis(state.settings.request_timeout_ms);
    let max_request_body_bytes = state.settings.max_request_body_bytes;

    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/api/v1/chat/ingestion", post(ingest))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http()) // ensures all requests are observable
                .layer(RequestBodyLimitLayer::new(max_request_body_bytes)) // prevents memory exhaustion
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .timeout(request_timeout), // prevents stuck work from consuming capacity
        )
        .layer(DefaultBodyLimit::disable())
        .with_state(state)
}

/// Liveness reflects only process health.
async fn healthz(State(state): State<AppState>) -> Json<Liveness> {
    Json(state.health.liveness("ingress"))
}

/// Readiness fails closed when dependencies are unsafe.
async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let readiness = state.health.readiness("ingress");

    if readiness.ok {
        (StatusCode::OK, Json(readiness))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(readiness))
    }
}

/// Ingestion enforces all correctness boundaries before accepting work.
async fn ingest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<IngestRequest>,
) -> Response {
    let request_id = header_or_uuid(&headers, "x-request-id");

    // Reject when durability cannot be guaranteed
    if !state.health.is_ready() {
        warn!(request_id, "rejecting: dependency unready");

        metrics::inc_rejected();

        return error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "dependency_unavailable",
            "service is not ready to accept events",
            &request_id,
        );
    }

    // Bound concurrent work to prevent resource exhaustion
    let _permit = match state.in_flight.clone().try_acquire_owned() {
        Ok(p) => p,
        Err(_) => {
            warn!(request_id, "rejecting: overloaded");

            metrics::inc_rejected();

            return error_response(
                StatusCode::TOO_MANY_REQUESTS,
                "overloaded",
                "request concurrency budget exhausted",
                &request_id,
            );
        }
    };

    // Reject invalid input before entering pipeline
    if let Err(msg) = validate_request(&request) {
        warn!(request_id, error = %msg, "rejecting: invalid request");

        metrics::inc_rejected();

        return error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_request",
            &msg,
            &request_id,
        );
    }

    // Normalize into stable event envelope
    let event = EventEnvelope::new(
        request.event_type,
        request.user_id,
        request.room_id,
        request.payload,
        request.producer_timestamp,
    );

    let event_id = event.event_id.clone();

    // Only acknowledge after broker durability
    match state.publisher.publish(&event).await {
        Ok(()) => {
            info!(request_id, event_id, "event accepted");

            metrics::inc_accepted();

            let mut response = (
                StatusCode::ACCEPTED,
                Json(IngestAccepted {
                    ok: true,
                    event_id,
                    request_id: request_id.clone(),
                }),
            )
                .into_response();

            response.headers_mut().insert(
                "x-request-id",
                HeaderValue::from_str(&request_id)
                    .unwrap_or_else(|_| HeaderValue::from_static("invalid")),
            );

            response
        }
        Err(err) => {
            error!(request_id, event_id, error = %err, "publish failed");

            metrics::inc_publish_failure();
            metrics::inc_rejected();

            error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "publish_failed",
                "failed to publish event",
                &request_id,
            )
        }
    }
}

/// Prevent malformed data from entering the system.
fn validate_request(request: &IngestRequest) -> Result<(), String> {
    if request.event_type.trim().is_empty() {
        return Err("event_type must be non-empty".into());
    }
    if request.user_id.trim().is_empty() {
        return Err("user_id must be non-empty".into());
    }
    if request.room_id.trim().is_empty() {
        return Err("room_id must be non-empty".into());
    }
    if !request.payload.is_object() {
        return Err("payload must be a JSON object".into());
    }
    Ok(())
}

/// Ensure every request has a correlation id.
fn header_or_uuid(headers: &HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

/// Standard failure response with traceability.
fn error_response(
    status: StatusCode,
    code: &'static str,
    message: &str,
    request_id: &str,
) -> Response {
    let mut response = (
        status,
        Json(ErrorBody {
            ok: false,
            code,
            message: message.to_string(),
            request_id: request_id.to_string(),
        }),
    )
        .into_response();

    response.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(request_id)
            .unwrap_or_else(|_| HeaderValue::from_static("invalid")),
    );

    response
}

/// Timeouts must fail explicitly to avoid hidden resource leaks.
async fn handle_timeout_error(_: BoxError) -> Response {
    error_response(
        StatusCode::REQUEST_TIMEOUT,
        "request_timeout",
        "request exceeded timeout budget",
        &Uuid::new_v4().to_string(),
    )
}

type BoxError = Box<dyn std::error::Error + Send + Sync>;