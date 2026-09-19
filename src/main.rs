use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use legal_agent_failure_tracker::{
    failure_client::{FailureClient, InfraiError},
    matter_loop::{evaluate, MatterRun, MatterStage},
};
use serde::Serialize;

#[derive(Clone)]
struct ServiceState {
    failures: FailureClient,
}

#[derive(Serialize)]
struct RunResult {
    matter_id: String,
    stage: MatterStage,
    failure_captured: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = ServiceState { failures: FailureClient::from_env()? };
    let app = Router::new().route("/run", post(run_matter)).with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("legal agent failure tracker listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn run_matter(
    State(state): State<ServiceState>,
    Json(run): Json<MatterRun>,
) -> Result<Json<RunResult>, (StatusCode, String)> {
    match evaluate(&run) {
        Ok(stage) => Ok(Json(RunResult {
            matter_id: run.matter_id,
            stage,
            failure_captured: false,
        })),
        Err(failure) => {
            state
                .failures
                .capture(&failure.to_string(), &failure.idempotency_key())
                .await
                .map_err(map_infrai_error)?;
            Ok(Json(RunResult {
                matter_id: run.matter_id,
                stage: failure.stage(),
                failure_captured: true,
            }))
        }
    }
}

fn map_infrai_error(error: InfraiError) -> (StatusCode, String) {
    match error {
        InfraiError::Rejected { status, code, message } if status.is_client_error() => {
            (status, format!("{code}: {message}"))
        }
        other => (StatusCode::BAD_GATEWAY, other.to_string()),
    }
}

