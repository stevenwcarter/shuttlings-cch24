use anyhow::{bail, Context};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use cargo_manifest::Manifest;
use hyper::HeaderMap;
use itertools::Itertools;
use serde_json::Value as JsonValue;
use toml::Table;

pub fn day_05_routes() -> Router {
    Router::new().route("/manifest", post(manifest_5))
}

fn convert_to_toml(input: &str, format: &str) -> anyhow::Result<String> {
    // Deserialize based on the input format
    let value: JsonValue = match format {
        "yaml" => serde_yaml::from_str(input).context("could not deserialize as yaml")?,
        "json" => serde_json::from_str(input).context("could not deserialize as json")?,
        _ => bail!("Unsupported format. Use 'yaml' or 'json'."),
    };

    // Serialize the generic `Value` to TOML
    let toml_string = toml::to_string(&value)?;

    Ok(toml_string)
}

fn bad_request(message: &str) -> Response {
    (StatusCode::BAD_REQUEST, message.to_string()).into_response()
}

fn unsupported_type(message: &str) -> Response {
    (StatusCode::UNSUPPORTED_MEDIA_TYPE, message.to_string()).into_response()
}

fn no_content() -> Response {
    (StatusCode::NO_CONTENT).into_response()
}
async fn manifest_5(headers: HeaderMap, data: String) -> Response {
    let data = match headers.get("Content-Type") {
        Some(header) => match header.to_str() {
            Ok("application/json") => convert_to_toml(&data, "json"),
            Ok("application/yaml") => convert_to_toml(&data, "yaml"),
            Ok("application/toml") | Ok("text/toml") => Ok(data),
            _ => return unsupported_type("Unknown content type"),
        },
        _ => return bad_request("No content type header"),
    };
    let data = match data {
        Ok(data) => data,
        _ => return bad_request("could not parse data"),
    };
    let manifest = match Manifest::from_slice(data.as_bytes()) {
        Ok(m) => m,
        Err(_) => return bad_request("Invalid manifest"),
    };

    // Validate the package and keywords
    let package = match manifest.package {
        Some(p) => p,
        None => return bad_request("Magic keyword not provided"),
    };

    let keywords = match package.keywords.and_then(|k| k.as_local()) {
        Some(k) => k,
        None => return bad_request("Magic keyword not provided"),
    };

    if !keywords.contains(&"Christmas 2024".to_string()) {
        return bad_request("Magic keyword not provided");
    }

    // Parse the data into a Table
    let data_table = match data.parse::<Table>() {
        Ok(table) => table,
        Err(_) => return bad_request("Failed to parse data into table"),
    };

    // Extract and process orders
    let parsed_orders = data_table
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("orders"))
        .and_then(|o| o.as_array())
        .map(|orders| {
            orders
                .iter()
                .filter_map(|o| {
                    let item = o.get("item").and_then(|i| i.as_str())?;
                    let quantity = o
                        .get("quantity")
                        .and_then(|q| q.as_integer())
                        .filter(|&q| q > 0)?;
                    Some((item.to_string(), quantity as u32))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // Return the appropriate response
    if parsed_orders.is_empty() {
        no_content()
    } else {
        let result = parsed_orders
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .join("\n");
        result.into_response()
    }
}

// async fn manifest_5(data: String) -> Response {
//     let manifest = Manifest::from_slice(data.as_bytes());
//     match manifest {
//         Ok(manifest) => {
//             if let Some(package) = manifest.package {
//                 if package.keywords.is_none()
//                     || package.keywords.clone().unwrap().as_local().is_none()
//                 {
//                     return (StatusCode::BAD_REQUEST, "Magic keyword not provided").into_response();
//                 }
//                 let keywords = package.keywords.unwrap().as_local().unwrap();
//                 if !keywords.contains(&"Christmas 2024".to_string()) {
//                     return (StatusCode::BAD_REQUEST, "Magic keyword not provided").into_response();
//                 }
//                 let data = data.parse::<Table>().unwrap();
//                 let mut parsed_orders: Vec<(String, u32)> = Vec::new();
//
//                 if let Some(package) = data.get("package") {
//                     if let Some(metadata) = package.get("metadata") {
//                         if let Some(orders) = metadata.get("orders") {
//                             if let Some(orders) = orders.as_array() {
//                                 orders.iter().for_each(|o| {
//                                     if let Some(order) = o.as_table() {
//                                         if let Some(item) = order.get("item") {
//                                             if let Some(item) = item.as_str() {
//                                                 if let Some(quantity) = order.get("quantity") {
//                                                     if quantity.is_integer() {
//                                                         if let Some(quantity) =
//                                                             quantity.as_integer()
//                                                         {
//                                                             if quantity > 0 {
//                                                                 parsed_orders.push((
//                                                                     item.to_string(),
//                                                                     quantity as u32,
//                                                                 ));
//                                                             }
//                                                         }
//                                                     }
//                                                 }
//                                             }
//                                         }
//                                     }
//                                 })
//                             }
//                         }
//                     }
//                 }
//
//                 if parsed_orders.is_empty() {
//                     (StatusCode::NO_CONTENT).into_response()
//                 } else {
//                     let result = parsed_orders
//                         .iter()
//                         .map(|(k, v)| format!("{k}: {v}"))
//                         .join("\n");
//                     result.into_response()
//                 }
//             } else {
//                 (StatusCode::BAD_REQUEST, "Magic keyword not provided").into_response()
//             }
//         }
//         _ => (StatusCode::BAD_REQUEST, "Invalid manifest").into_response(),
//     }
// }
