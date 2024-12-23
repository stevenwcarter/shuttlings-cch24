use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use cargo_manifest::Manifest;
use itertools::Itertools;
use toml::Table;

pub async fn manifest_5(data: String) -> Response {
    let manifest = Manifest::from_slice(data.as_bytes());
    match manifest {
        Ok(manifest) => {
            if let Some(package) = manifest.package {
                if package.keywords.is_none()
                    || package.keywords.clone().unwrap().as_local().is_none()
                {
                    return (StatusCode::BAD_REQUEST, "Magic keyword not provided").into_response();
                }
                let keywords = package.keywords.unwrap().as_local().unwrap();
                if !keywords.contains(&"Christmas 2024".to_string()) {
                    return (StatusCode::BAD_REQUEST, "Magic keyword not provided").into_response();
                }
                let data = data.parse::<Table>().unwrap();
                let mut parsed_orders: Vec<(String, u32)> = Vec::new();

                if let Some(package) = data.get("package") {
                    if let Some(metadata) = package.get("metadata") {
                        if let Some(orders) = metadata.get("orders") {
                            if let Some(orders) = orders.as_array() {
                                orders.iter().for_each(|o| {
                                    if let Some(order) = o.as_table() {
                                        if let Some(item) = order.get("item") {
                                            if let Some(item) = item.as_str() {
                                                if let Some(quantity) = order.get("quantity") {
                                                    if quantity.is_integer() {
                                                        if let Some(quantity) =
                                                            quantity.as_integer()
                                                        {
                                                            if quantity > 0 {
                                                                parsed_orders.push((
                                                                    item.to_string(),
                                                                    quantity as u32,
                                                                ));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                })
                            }
                        }
                    }
                }

                if parsed_orders.is_empty() {
                    (StatusCode::NO_CONTENT).into_response()
                } else {
                    let result = parsed_orders
                        .iter()
                        .map(|(k, v)| format!("{k}: {v}"))
                        .join("\n");
                    result.into_response()
                }
            } else {
                (StatusCode::BAD_REQUEST, "Magic keyword not provided").into_response()
            }
        }
        _ => (StatusCode::BAD_REQUEST, "Invalid manifest").into_response(),
    }
}
