use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query};
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
        .layer(Extension(pool))
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
