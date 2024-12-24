use axum::{
    http::{self, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use shuttlings_cch24::{day12::day_12_routes, day16::day_16_routes, day5::*};
use shuttlings_cch24::{day2::*, day9::day_9_routes};
use tower_http::trace::TraceLayer;

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
async fn main() -> shuttle_axum::ShuttleAxum {
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
                        span.record("status_code", tracing::field::display(response.status()));
                    },
                ),
        );

    Ok(router.into())
}
