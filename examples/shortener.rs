use anyhow::Result;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use http::{header::LOCATION, HeaderMap, StatusCode};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[derive(Debug, Deserialize)]
struct ShortenReq {
    url: String,
}

#[derive(Debug, Serialize)]
struct ShortenRes {
    url: String,
}

#[derive(Debug, Clone)]
struct AppState {
    db: PgPool,
}

// FromRow 是 sqlx 库中的一个宏，允许你将数据库中的一行数据直接映射到一个结构体实例
#[derive(Debug, FromRow)]
struct UrlRecord {
    // 当结构包含查询中不存在的字段时，如果字段类型具有 Default 的实现，则可以使用 default 属性为所述字段分配默认值。
    #[sqlx(default)]
    id: String,
    #[sqlx(default)]
    url: String,
}

const LISTEN_ADDR: &str = "127.0.0.1:9876";

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let url = "postgres://postgres:postgres@localhost:5432/shortener";
    let state = AppState::try_new(url).await?;
    info!("Connected to database: {url}");
    let listener = TcpListener::bind(LISTEN_ADDR).await?;
    info!("Listening on: {LISTEN_ADDR}");

    let app = Router::new()
        .route("/", post(shorten))
        .route("/:id", get(redirect))
        .with_state(state);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

// 这两个写法是 Rust 中模式匹配（pattern matching）的一部分，结合了提取器（extractors）和解构（destructuring）的使用方法，
// 主要用于处理请求参数。在 Axum 框架中，这种写法用于从请求中提取状态和请求体中的 JSON 数据。
// State(state)：State 是一个提取器，用于从请求的扩展（extensions）中提取共享状态。在这里，state 是提取后的变量名称。
// State<AppState>：这里指定了提取器的类型，即 State<AppState>。它表示提取器将提取一个 AppState 类型的共享状态
// Json(data)：Json 是一个提取器，用于从请求体中提取并反序列化 JSON 数据。在这里，data 是提取后的变量名称
// Json<ShortenReq>：这里指定了提取器的类型，即 Json<ShortenReq>。它表示提取器将请求体中的 JSON 数据反序列化为 ShortenReq 类型的实例
async fn shorten(
    State(state): State<AppState>,
    Json(data): Json<ShortenReq>,
    // Result<impl IntoResponse, StatusCode> 实际上只有一个返回值，但这个返回值可能是两种不同的情况之一。
    // Result<T, E> 是 Rust 的一个枚举类型，用于表示可能成功或失败的操作。它有两个变体：
    // Ok(T)：表示操作成功，包含类型 T 的值，在这里 T 是 impl IntoResponse。
    // Err(E)：表示操作失败，包含类型 E 的错误，在这里 E 是 StatusCode。
    // 所以，这个函数实际上返回的是一个单一的 Result 值，但这个 Result 可能是以下两种情况之一：
    // 成功情况：Ok(impl IntoResponse)：这里的 impl IntoResponse 表示任何实现了 IntoResponse trait 的类型。在您之前的代码中，这通常是 (StatusCode, Json<ShortenRes>) 的形式。
    // 错误情况：Err(StatusCode)：这表示一个错误，使用 HTTP 状态码来表示错误的类型。
) -> Result<impl IntoResponse, StatusCode> {
    let id = state.shorten(&data.url).await.map_err(|e| {
        warn!("Failed to shorten URL: {e}");
        StatusCode::UNPROCESSABLE_ENTITY
    })?;
    let body = Json(ShortenRes {
        url: format!("http://{}/{}", LISTEN_ADDR, id),
    });
    // Axum 为元组 (StatusCode::CREATED, body) 实现了 IntoResponse trait。
    // IntoResponse 提供了一个统一的接口，用于将各种类型转换成 HTTP 响应。
    Ok((StatusCode::CREATED, body))
}

async fn redirect(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let url = state
        .get_url(&id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, url.parse().unwrap());
    Ok((StatusCode::PERMANENT_REDIRECT, headers))
}

impl AppState {
    async fn try_new(url: &str) -> Result<Self> {
        let pool = PgPool::connect(url).await?;
        // Create table if not exists
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS urls (
                id CHAR(6) PRIMARY KEY,
                url TEXT NOT NULL UNIQUE
            )
            "#,
        )
        .execute(&pool)
        .await?;
        Ok(Self { db: pool })
    }

    async fn shorten(&self, url: &str) -> Result<String> {
        let id = nanoid!(6);
        // ON CONFLICT(url)：这部分指定在插入过程中，如果在 url 列上发生冲突（即 url 列中已存在相同的值），应如何处理
        // DO UPDATE SET url=EXCLUDED.url：当发生冲突时，不是简单地忽略或报错，而是执行更新操作。
        // EXCLUDED 是一个特殊的表别名，代表正在尝试插入的那一行。
        // SET url=EXCLUDED.url 表示将现有行的 url 列更新为冲突的那一行的 url 值（虽然在这种情况下，值是相同的，因此实际效果是保持不变）。
        let ret: UrlRecord = sqlx::query_as(
            "INSERT INTO urls (id, url) VALUES ($1, $2) ON CONFLICT(url) DO UPDATE SET url=EXCLUDED.url RETURNING id",
        ).bind(id)
            .bind(url).fetch_one(&self.db).await?;
        Ok(ret.id)
    }

    async fn get_url(&self, id: &str) -> Result<String> {
        let ret: UrlRecord = sqlx::query_as("SELECT url FROM urls WHERE id = $1")
            .bind(id)
            .fetch_one(&self.db)
            .await?;
        Ok(ret.url)
    }
}
