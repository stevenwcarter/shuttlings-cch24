use itertools::Itertools;
use std::io::Read;

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use axum_extra::extract::Multipart;
use toml::Table;

pub fn day_23_routes() -> Router {
    Router::new()
        .route("/star", get(star))
        .route("/present/:color", get(present_color))
        .route("/ornament/:state/:n", get(ornament))
        .route("/lockfile", post(lockfile))
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
        "<div class='ornament{}' id='ornament{n}' hx-trigger='load delay:2s once' hx-get='/23/ornament/{next_state}/{n}' hx-swap='outerHTML'></div>",
        class
    );
    println!("Result: '{result}'");

    (StatusCode::OK, result)
}

pub async fn lockfile(mut multipart: Multipart) -> Response {
    if let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        println!("Length of `{}` is {} bytes", name, data.len());
        let data = String::from_utf8(data.into()).unwrap();
        println!("Data: {data}");
        match data.parse::<Table>() {
            Ok(data) => {
                if let Some(packages) = data.get("package") {
                    println!("Packges: {:#?}", packages);
                    let checksums: Vec<_> = packages
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter_map(|p| p.as_table())
                        .inspect(|a| println!("{:?}", a))
                        .filter_map(|p| p.get("checksum"))
                        .collect();
                    for checksum in checksums.clone() {
                        if !checksum.is_str() {
                            return StatusCode::BAD_REQUEST.into_response();
                        }
                        if checksum.as_str().unwrap().len() < 10 {
                            return StatusCode::UNPROCESSABLE_ENTITY.into_response();
                        }
                        if !checksum
                            .as_str()
                            .unwrap()
                            .chars()
                            .all(|c| c.is_ascii_hexdigit())
                        {
                            return StatusCode::UNPROCESSABLE_ENTITY.into_response();
                        }
                    }
                    let data = checksums.iter()
                        .filter(|c| c.is_str())
                        .inspect(|a| println!("{:?}", a))
                        .filter_map(|c| c.as_str())
                        .inspect(|a| println!("{:?}", a))
                        .map(|c| {
                            let color = &c[0..6];
                            let top = i64::from_str_radix(&c[6..8], 16).unwrap();
                            let left = i64::from_str_radix(&c[8..10], 16).unwrap();

                            format!("<div style='background-color:#{color};top:{top}px;left:{left}px;'></div>")
                        })
                        .inspect(|a| println!("{:?}", a))
                        .collect::<Vec<String>>()
                        .join("");
                    (StatusCode::OK, data).into_response()
                } else {
                    println!("No package data");
                    (StatusCode::BAD_REQUEST, "".to_owned()).into_response()
                }
            }
            _ => (StatusCode::BAD_REQUEST, "".to_owned()).into_response(),
        }
    } else {
        (StatusCode::BAD_REQUEST, "ok".to_owned()).into_response()
    }
}
