use axum::{
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use shuttlings_cch24::day5::*;
use shuttlings_cch24::{day2::*, day9::day_9_routes};

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
        .nest("/9", day_9_routes());

    Ok(router.into())
}
