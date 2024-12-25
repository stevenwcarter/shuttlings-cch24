use std::convert::Infallible;

use axum::{
    body::{to_bytes, Body, Bytes},
    http::{self, HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Router,
};
use hyper::Request;
use shuttlings_cch24::{
    day12::day_12_routes, day16::day_16_routes, day19::day_19_routes, day23::day_23_routes, day5::*,
};
use shuttlings_cch24::{day2::*, day9::day_9_routes};
use tower_http::{body::Full, services::ServeDir, trace::TraceLayer};

async fn hello_world() -> &'static str {
    "Hello, bird!"
}

async fn seek_negative_one() -> impl IntoResponse {
    let mut headers: HeaderMap = HeaderMap::new();

    headers.insert(
        "Location",
        "https://www.youtube.com/watch?v=9Gc4QTqslN4"
            .parse()
            .unwrap(),
    );

    (StatusCode::FOUND, headers)
}

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: sqlx::PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    let router = Router::new()
        .route("/", get(hello_world))
        .route("/2/dest", get(dest_2))
        .route("/2/key", get(key_2))
        .route("/2/v6/dest", get(dest_2_v6))
        .route("/2/v6/key", get(key_2_v6))
        .route("/5/manifest", post(manifest_5))
        .route("/-1/seek", get(seek_negative_one))
        .nest("/9", day_9_routes())
        .nest("/12", day_12_routes())
        .nest("/16", day_16_routes())
        .nest("/23", day_23_routes())
        .nest("/19", day_19_routes(pool.clone()))
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(Extension(pool))
        // .layer(axum::middleware::from_fn(log_response_middleware))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &http::Request<_>| {
                    print!("Method: {} ", request.method());
                    println!("URI: {}", request.uri());
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        status_code = tracing::field::Empty,
                    )
                })
                .on_response(
                    |response: &http::Response<_>,
                     _latency: std::time::Duration,
                     span: &tracing::Span| {
                        println!("StatusCode: {}", response.status());
                        // println!("Body: {:?}", response.body());
                        span.record("status_code", tracing::field::display(response.status()));
                    },
                ),
        );

    Ok(router.into())
}

// async fn log_response_middleware(req: Request<axum::body::Body>, next: Next) -> impl IntoResponse {
//     // Proceed with the request
//     let mut response = next.run(req).await;
//
//     // Extract and log the response body
//     if let Ok(bytes) = to_bytes(*response.body(), 200000).await {
//         let body_string = String::from_utf8_lossy(&bytes);
//         println!("Response body: {}", body_string);
//
//         // Reconstruct the response body since it was consumed
//         let new_body = Response::from_parts(_, body)
//         response = response.map(|_| new_body);
//     }
//
//     response
// }
