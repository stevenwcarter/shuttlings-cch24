use anyhow::{Context, Result};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use base64::prelude::*;
use core::str;
use jsonwebtoken::{
    decode, decode_header, encode, errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header,
    Validation,
};
use pem::parse;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashSet,
    fs::{read, read_to_string},
};

const SECRET_KEY: &[u8] = b"28348932";

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    data: String,
    exp: usize,
}

// #[derive(Debug, Serialize, Deserialize)]
// struct SantaClaims {
//
// }

pub fn day_16_routes() -> Router {
    Router::new()
        .route("/wrap", post(wrap))
        .route("/unwrap", get(unwrap))
        .route("/decode", post(decode_path))
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
pub async fn decode_path(token: String) -> Response {
    let header = match decode_header(&token) {
        Ok(header) => {
            println!("Header: {:?}", header);
            header
        }
        _ => return (StatusCode::BAD_REQUEST, "invalid header".to_owned()).into_response(),
    };
    println!("JWT Algorithm: {:?}", header.alg);
    let public_key_pem = read_to_string("assets/day16_santa_public_key.pem").unwrap();
    let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes()).unwrap();

    let mut validation = Validation::new(header.alg);
    validation.validate_exp = false;
    validation.required_spec_claims = HashSet::new();

    let token = decode::<serde_json::Value>(&token, &decoding_key, &validation);
    match token {
        Ok(token) => {
            let claims = token.claims;
            if claims.is_object() {
                (StatusCode::OK, Json(claims)).into_response()
            } else {
                (StatusCode::BAD_REQUEST, format!("Bad claim: {:?}", claims)).into_response()
            }
        }
        Err(ref e) if *e.kind() == ErrorKind::InvalidSignature => {
            (StatusCode::UNAUTHORIZED, "invalid signature".to_owned()).into_response()
        }
        Err(e) => {
            println!("Error: {:?}", e);
            (StatusCode::BAD_REQUEST, "invalid signature".to_owned()).into_response()
        }
    }
}
