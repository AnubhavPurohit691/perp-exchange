use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::model::Candle;

#[derive(Debug, serde::Deserialize)]
pub struct CandlesQuery {
    pub symbol: String,
    pub timeframe: String,
    #[serde(default)]
    pub from: Option<DateTime<Utc>>,
    #[serde(default)]
    pub to: Option<DateTime<Utc>>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    500
}

/// GET /candles?symbol=btc&timeframe=1&from=...&to=...&limit=500
pub async fn get_candles(
    Extension(pool): Extension<PgPool>,
    Query(params): Query<CandlesQuery>,
) -> impl IntoResponse {
    let limit = params.limit.clamp(1, 1000);

    let rows: Result<Vec<Candle>, sqlx::Error> = if params.from.is_some() || params.to.is_some() {
        sqlx::query_as::<_, Candle>(
            r#"
            SELECT symbol, timeframe, bucket_start, open, high, low, close,
                   volume, trade_count, open_ts, close_ts
            FROM candles
            WHERE symbol = $1 AND timeframe = $2
              AND ($3::timestamptz IS NULL OR bucket_start >= $3)
              AND ($4::timestamptz IS NULL OR bucket_start <= $4)
            ORDER BY bucket_start ASC
            LIMIT $5
            "#,
        )
        .bind(&params.symbol)
        .bind(&params.timeframe)
        .bind(params.from)
        .bind(params.to)
        .bind(limit)
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query_as::<_, Candle>(
            r#"
            SELECT symbol, timeframe, bucket_start, open, high, low, close,
                   volume, trade_count, open_ts, close_ts
            FROM candles
            WHERE symbol = $1 AND timeframe = $2
            ORDER BY bucket_start DESC
            LIMIT $3
            "#,
        )
        .bind(&params.symbol)
        .bind(&params.timeframe)
        .bind(limit)
        .fetch_all(&pool)
        .await
    };

    match rows {
        Ok(candles) => {
            let out: Vec<Candle> = if params.from.is_some() || params.to.is_some() {
                candles
            } else {
                candles.into_iter().rev().collect()
            };
            Json(out).into_response()
        }
        Err(e) => {
            eprintln!("candles query error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch candles").into_response()
        }
    }
}
