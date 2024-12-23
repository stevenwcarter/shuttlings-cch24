use std::{default, sync::Arc, time::Duration};

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use leaky_bucket::RateLimiter;
use serde::{Deserialize, Serialize};

pub struct CustomLimiter {
    pub rate_limiter: RateLimiter,
}

pub fn day_9_routes() -> Router {
    Router::new()
        .route("/milk", post(milk))
        // .route("/refill", post(refill))
        .with_state(build_milk_limiter())
}

pub fn build_milk_limiter() -> Arc<RateLimiter> {
    Arc::new(
        RateLimiter::builder()
            .max(5)
            .initial(5)
            .interval(Duration::from_secs(1))
            .refill(1)
            .build(),
    )
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub struct Day9Json {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liters: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gallons: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub litres: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pints: Option<f32>,
}

pub async fn milk(
    State(rate_limiter): State<Arc<RateLimiter>>,
    headers: HeaderMap,
    payload: Option<Json<Day9Json>>,
) -> Response {
    println!("\nHeaders: {:#?}\nPayload: {:#?}", headers, payload);
    match rate_limiter.try_acquire(1) {
        true => {
            if let Some(content_type) = headers.get("Content-Type") {
                if content_type == "application/json" {
                    if let Some(payload) = payload {
                        match (
                            payload.liters,
                            payload.gallons,
                            payload.litres,
                            payload.pints,
                        ) {
                            (Some(liters), None, None, None) => {
                                let result = Day9Json {
                                    gallons: Some(liters * 0.26417205),
                                    ..Day9Json::default()
                                };
                                (StatusCode::OK, Json(result)).into_response()
                            }
                            (None, Some(gallons), None, None) => {
                                let result = Day9Json {
                                    liters: Some(gallons / 0.26417205),
                                    ..Day9Json::default()
                                };
                                (StatusCode::OK, Json(result)).into_response()
                            }
                            (None, None, Some(litres), None) => {
                                let result = Day9Json {
                                    pints: Some(litres * 1.759754),
                                    ..Day9Json::default()
                                };
                                (StatusCode::OK, Json(result)).into_response()
                            }
                            (None, None, None, Some(pints)) => {
                                let result = Day9Json {
                                    litres: Some(pints / 1.759754),
                                    ..Day9Json::default()
                                };
                                (StatusCode::OK, Json(result)).into_response()
                            }
                            _ => (StatusCode::BAD_REQUEST).into_response(),
                        }
                    } else {
                        (StatusCode::BAD_REQUEST, "").into_response()
                    }
                } else {
                    (StatusCode::OK, "Milk withdrawn\n").into_response()
                }
            } else {
                (StatusCode::OK, "Milk withdrawn\n").into_response()
            }
        }
        false => (StatusCode::TOO_MANY_REQUESTS, "No milk available\n").into_response(),
    }
}

pub async fn refill(State(limiter): State<RateLimiter>) -> impl IntoResponse {
    (StatusCode::OK, "")
}
