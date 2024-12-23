use std::net::Ipv6Addr;

use axum::{extract::Query, response::IntoResponse};
use itertools::Itertools;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Day2Params {
    from: String,
    key: Option<String>,
    to: Option<String>,
}
pub async fn dest_2(params: Query<Day2Params>) -> impl IntoResponse {
    let params = params.0;
    let from: Vec<u8> = params
        .from
        .split('.')
        .map(|p| p.parse::<u8>().unwrap())
        .collect();
    let key: Vec<u8> = params
        .key
        .unwrap()
        .split('.')
        .map(|p| p.parse::<u8>().unwrap())
        .collect();

    (0..from.len())
        .map(|i| from[i].wrapping_add(key[i]))
        .join(".")
}
pub async fn dest_2_v6(params: Query<Day2Params>) -> impl IntoResponse {
    let params = params.0;
    let from: Ipv6Addr = params.from.parse().unwrap();
    let key: Ipv6Addr = params.key.unwrap().parse().unwrap();

    let result = from.to_bits() ^ key.to_bits();

    let result: Ipv6Addr = Ipv6Addr::from(result);

    result.to_string()
}
pub async fn key_2(params: Query<Day2Params>) -> impl IntoResponse {
    let params = params.0;
    let from: Vec<u8> = params
        .from
        .split('.')
        .map(|p| p.parse::<u8>().unwrap())
        .collect();
    let to: Vec<u8> = params
        .to
        .unwrap()
        .split('.')
        .map(|p| p.parse::<u8>().unwrap())
        .collect();

    (0..from.len())
        .map(|i| to[i].wrapping_sub(from[i]))
        .join(".")
}
pub async fn key_2_v6(params: Query<Day2Params>) -> impl IntoResponse {
    let params = params.0;
    let from: Ipv6Addr = params.from.parse().unwrap();
    let to: Ipv6Addr = params.to.unwrap().parse().unwrap();

    let result = from.to_bits() ^ to.to_bits();

    let result: Ipv6Addr = Ipv6Addr::from(result);

    result.to_string()
}
