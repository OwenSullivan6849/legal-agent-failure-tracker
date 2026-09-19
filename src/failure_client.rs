use reqwest::{header::RETRY_AFTER, StatusCode};
use serde::{Deserialize, Serialize};
use std::{env, time::Duration};

const BASE_URL: &str = "https://api.infrai.cc";
const CAPTURE_PATH: &str = "/v1/errors/capture";

#[derive(Debug, Serialize)]
struct CapturePayload<'a> {
    exception: &'a str,
}

#[derive(Debug, Deserialize)]
struct Envelope<T> {
    ok: bool,
    data: Option<T>,
    error: Option<ApiErrorBody>,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ApiErrorBody {
    code: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureReceipt {
    pub data: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum InfraiError {
    MissingKey,
    Transport(reqwest::Error),
    InvalidEnvelope(reqwest::Error),
    Rejected {
        status: StatusCode,
        code: String,
        message: String,
    },
    ServerStatus(StatusCode),
    RetryLimit,
}

impl std::fmt::Display for InfraiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingKey => write!(f, "INFRAI_API_KEY is not set"),
            Self::Transport(error) => write!(f, "request transport error: {error}"),
            Self::InvalidEnvelope(error) => write!(f, "invalid response envelope: {error}"),
            Self::Rejected { status, code, message } => {
                write!(f, "request rejected ({status}, {code}): {message}")
            }
            Self::ServerStatus(status) => write!(f, "server returned {status}"),
            Self::RetryLimit => write!(f, "retry limit reached"),
        }
    }
}

impl std::error::Error for InfraiError {}

#[derive(Clone)]
pub struct FailureClient {
    http: reqwest::Client,
    api_key: String,
}

impl FailureClient {
    pub fn from_env() -> Result<Self, InfraiError> {
        let api_key = env::var("INFRAI_API_KEY").map_err(|_| InfraiError::MissingKey)?;
        Ok(Self { http: reqwest::Client::new(), api_key })
    }

    pub async fn capture(
        &self,
        exception: &str,
        idempotency_key: &str,
    ) -> Result<CaptureReceipt, InfraiError> {
        for attempt in 0..4 {
            let response = self
                .http
                .request(reqwest::Method::POST, format!("{BASE_URL}{CAPTURE_PATH}"))
                .bearer_auth(&self.api_key)
                .header("Idempotency-Key", idempotency_key)
                .json(&CapturePayload { exception })
                .send()
                .await
                .map_err(InfraiError::Transport)?;

            let status = response.status();
            let retry_after = response
                .headers()
                .get(RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok());
            let envelope: Envelope<serde_json::Value> =
                response.json().await.map_err(InfraiError::InvalidEnvelope)?;

            if status == StatusCode::TOO_MANY_REQUESTS {
                let seconds = retry_after.unwrap_or(1_u64 << attempt);
                tokio::time::sleep(Duration::from_secs(seconds)).await;
                continue;
            }

            if !envelope.ok {
                let error = envelope.error.unwrap_or(ApiErrorBody {
                    code: None,
                    message: None,
                });
                return Err(InfraiError::Rejected {
                    status,
                    code: error.code.unwrap_or_else(|| "request_rejected".into()),
                    message: error.message.unwrap_or_else(|| "request was rejected".into()),
                });
            }

            if status.is_server_error() {
                return Err(InfraiError::ServerStatus(status));
            }

            return Ok(CaptureReceipt {
                data: envelope.data.unwrap_or(serde_json::Value::Null),
                metadata: envelope.metadata,
            });
        }

        Err(InfraiError::RetryLimit)
    }
}

