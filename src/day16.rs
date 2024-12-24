use anyhow::{Context, Result};
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use base64::prelude::*;
use core::str;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

const SECRET_KEY: &[u8] = b"28348932";

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    data: String,
    exp: usize,
}

pub fn day_16_routes() -> Router {
    Router::new()
        .route("/wrap", post(wrap))
        .route("/unwrap", get(unwrap))
}

pub async fn wrap(jar: CookieJar, data: String) -> impl IntoResponse {
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::minutes(60))
        .expect("valid timestamp")
        .timestamp() as usize;
    println!("Data: {data}");
    let claim = Claims { data, exp };

    let token = encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(SECRET_KEY),
    )
    .unwrap();

    (StatusCode::OK, jar.add(Cookie::new("gift", token)))
}

pub async fn unwrap(jar: CookieJar) -> impl IntoResponse {
    let gift = jar.get("gift");

    println!("Gift cookie: {:?}", gift);
    match gift {
        Some(gift) => {
            let token = decode::<Claims>(
                gift.value(),
                &DecodingKey::from_secret(SECRET_KEY),
                &Validation::default(),
            );
            match token {
                Ok(gift) => {
                    println!("Token: {:?}", gift.claims);
                    (StatusCode::OK, gift.claims.data)
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                    (StatusCode::BAD_REQUEST, "invalid encoding".to_owned())
                }
            }
        }
        None => (StatusCode::BAD_REQUEST, "no gift header found".to_owned()),
    }
}

// async fn encode(input: &str) -> String {
//     BASE64_STANDARD.encode(input)
// }
//
// async fn decode(input: &str) -> Result<String> {
//     let result = BASE64_STANDARD.decode(input).context("could not decode")?;
//
//     let result = str::from_utf8(&result).context("could not convert to utf8 string")?;
//
//     Ok(result.to_owned())
// }
