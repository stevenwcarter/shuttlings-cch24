use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

pub fn day_23_routes() -> Router {
    Router::new()
        .route("/star", get(star))
        .route("/present/:color", get(present_color))
        .route("/ornament/:state/:n", get(ornament))
}

pub async fn star() -> impl IntoResponse {
    "<div id='star' class='lit'></div>"
}

pub async fn present_color(Path(color): Path<String>) -> impl IntoResponse {
    let colors = ["red", "blue", "purple"];

    if !colors.contains(&color.as_str()) {
        return (StatusCode::IM_A_TEAPOT, "".to_string());
    }

    let current_index = colors.iter().position(|&c| c == color).unwrap();
    let next_index = (current_index + 1 + colors.len()) % colors.len();

    let element = format!(
        "<div class='present {}' hx-get='/23/present/{}' hx-swap='outerHTML'>
<div class='ribbon'></div>
<div class='ribbon'></div>
<div class='ribbon'></div>
<div class='ribbon'></div>
</div>",
        colors[current_index], colors[next_index]
    );

    (StatusCode::OK, element)
}

pub async fn ornament(Path((state, n)): Path<(String, String)>) -> impl IntoResponse {
    let valid_states = ["on", "off"];

    if !valid_states.contains(&state.as_str()) {
        return (StatusCode::IM_A_TEAPOT, "".to_string());
    }

    let n = html_escape::encode_text(n.as_str());

    let class = if state.as_str() == "on" { " on" } else { "" };
    let next_state = if state.as_str() == "on" { "off" } else { "on" };

    let result = format!(
        r#"<div class="ornament{}" id="ornament{n}" hx-trigger="load changed delay:2s once" hx-get="/23/ornament/{next_state}/{n}" hx-swap="outerHTML"></div>"#,
        class
    );

    (StatusCode::OK, result)
}
