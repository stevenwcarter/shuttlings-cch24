use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use hashbrown::HashMap;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct QuoteNew {
    pub author: String,
    pub quote: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, FromRow)]
pub struct Quote {
    pub id: Uuid,
    pub author: String,
    pub quote: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    version: i32,
}

pub fn day_19_routes(pool: sqlx::PgPool) -> Router {
    Router::new()
        .route("/reset", post(reset))
        .route("/cite/:id", get(get_quote_by_id))
        .route("/remove/:id", delete(remove_quote_by_id))
        .route("/undo/:id", put(undo))
        .route("/draft", post(draft))
        .route("/list", get(list))
        .layer(Extension(pool))
        .layer(Extension(Arc::new(Mutex::new(
            HashMap::<String, u32>::new(),
        ))))
}

fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    (0..16).map(|_| rng.sample(Alphanumeric) as char).collect()
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub token: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct List {
    quotes: Vec<Quote>,
    page: u32,
    next_token: Option<String>,
}

#[axum::debug_handler]
pub async fn list(
    Query(list_query): Query<ListQuery>,
    Extension(pool): Extension<sqlx::PgPool>,
    Extension(list_cache): Extension<Arc<Mutex<HashMap<String, u32>>>>,
) -> Response {
    let token = list_query.token;
    let mut page_number = 1;
    if let Some(token) = token {
        let list_cache = list_cache.lock().unwrap();
        let search = list_cache.get(&token);
        if let Some(search) = search {
            page_number = *search;
        } else {
            return (StatusCode::BAD_REQUEST).into_response();
        }
    }

    let offset = page_number.saturating_sub(1) * 3;
    match sqlx::query_as::<_, Quote>(
        "SELECT * FROM quotes ORDER BY created_at ASC LIMIT 4 OFFSET $1",
    )
    .bind(offset as i32)
    .fetch_all(&pool)
    .await
    {
        Ok(quotes_4) => {
            if quotes_4.len() == 4 {
                let quotes = quotes_4[0..4].to_vec();
                let token = generate_token();
                list_cache
                    .lock()
                    .unwrap()
                    .insert(token.to_owned(), page_number + 1);

                (
                    StatusCode::OK,
                    Json(List {
                        quotes,
                        page: page_number,
                        next_token: Some(token),
                    }),
                )
                    .into_response()
            } else {
                (
                    StatusCode::OK,
                    Json(List {
                        quotes: quotes_4.clone(),
                        page: page_number,
                        next_token: None,
                    }),
                )
                    .into_response()
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
            (StatusCode::BAD_REQUEST, Json("invalid input")).into_response()
        }
    }
}

pub async fn get_quote_by_id(
    Path(id): Path<Uuid>,
    Extension(pool): Extension<sqlx::PgPool>,
) -> Response {
    match sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
    {
        Ok(quote) => (StatusCode::OK, Json(quote)).into_response(),
        _ => (StatusCode::NOT_FOUND).into_response(),
    }
}
pub async fn remove_quote_by_id(
    Path(id): Path<Uuid>,
    Extension(pool): Extension<sqlx::PgPool>,
) -> Response {
    let query_result = sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await;
    if query_result.is_err() {
        return (StatusCode::NOT_FOUND).into_response();
    }
    let query = query_result.unwrap();
    match sqlx::query("DELETE FROM quotes WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(query)).into_response(),
        _ => (StatusCode::NOT_FOUND).into_response(),
    }
}
pub async fn undo(
    Path(id): Path<Uuid>,
    Extension(pool): Extension<sqlx::PgPool>,
    Json(quote): Json<QuoteNew>,
) -> Response {
    match sqlx::query("UPDATE quotes SET quote=$1,author=$2,version=version+1  WHERE id = $3")
        .bind(quote.quote.as_str())
        .bind(quote.author.as_str())
        .bind(id)
        .execute(&pool)
        .await
    {
        Ok(qr) => {
            if qr.rows_affected() == 0 {
                (StatusCode::NOT_FOUND).into_response()
            } else {
                get_quote_by_id(Path(id), Extension(pool)).await
            }
        }
        _ => (StatusCode::NOT_FOUND).into_response(),
    }
}
pub async fn draft(
    Extension(pool): Extension<sqlx::PgPool>,
    Json(quote): Json<QuoteNew>,
) -> Response {
    let id = uuid::Uuid::new_v4();
    let quote = Quote {
        id,
        author: quote.author,
        quote: quote.quote,
        version: 1,
        created_at: chrono::offset::Utc::now(),
    };
    match sqlx::query(
        "INSERT INTO quotes (id, author, quote, created_at, version) VALUES ($1, $2, $3, $4,$5)",
    )
    .bind(quote.id)
    .bind(quote.author)
    .bind(quote.quote)
    .bind(quote.created_at)
    .bind(quote.version)
    .execute(&pool)
    .await
    {
        Ok(_) => {
            match sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE id = $1")
                .bind(id)
                .fetch_one(&pool)
                .await
            {
                Ok(quote) => (StatusCode::CREATED, Json(quote)).into_response(),
                _ => (StatusCode::NOT_FOUND).into_response(),
            }
        }
        _ => (StatusCode::NOT_FOUND).into_response(),
    }
}

// pub id: Uuid,
// pub author: String,
// pub quote: String,
// pub created_at: chrono::DateTime<chrono::Utc>,
// version: i32,
pub async fn reset(Extension(pool): Extension<sqlx::PgPool>) -> impl IntoResponse {
    match sqlx::query("DELETE FROM quotes").execute(&pool).await {
        Ok(_) => (StatusCode::OK).into_response(),
        _ => (StatusCode::NOT_FOUND).into_response(),
    }
}
