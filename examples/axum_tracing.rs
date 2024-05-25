use std::time::Duration;

use axum::{routing::get, Router};
use tokio::{
    net::TcpListener,
    time::{sleep, Instant},
};
use tracing::{debug, info, instrument, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建每日滚动的日志文件附加器
    let file_appender = tracing_appender::rolling::daily("/tmp/logs", "ecosystem.log");
    // 将文件附加器设置为非阻塞模式
    // 非阻塞模式的优势在于日志记录操作不会阻塞应用程序的主要执行路径，提高了应用程序的性能和响应速度。
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // 使用 fmt::Layer 配置日志格式化。
    // .with_span_events(FmtSpan::CLOSE) 表示在跟踪 span 结束时记录日志，
    // .pretty() 表示使用美化的日志格式，
    // .with_filter(LevelFilter::DEBUG) 设置日志级别为 DEBUG。
    let console = fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::DEBUG);

    let file = fmt::Layer::new()
        .with_writer(non_blocking)
        .pretty()
        .with_filter(LevelFilter::INFO);

    // 注册控制台和文件日志。
    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .init();

    let addr = "0.0.0.0:8080";
    let app = Router::new().route("/", get(index_handler));

    let listener = TcpListener::bind(addr).await?;
    info!("Starting server on {}", addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

// #[instrument] 用于自动为函数添加跟踪span。
// 会加上函数的调用信息
// in axum_tracing::long_task
// in axum_tracing::index_handler
#[instrument]
async fn index_handler() -> String {
    debug!("index handler started");
    // await 表示等待一个持续 10 毫秒的异步睡眠操作完成。
    // 在这 10 毫秒内，函数会让出控制权，允许其他任务执行。
    sleep(Duration::from_millis(10)).await;
    let ret = long_task().await;
    info!(http.status = 200, "index handler completed");
    ret
}

#[instrument]
async fn long_task() -> String {
    let start = Instant::now();
    sleep(Duration::from_millis(112)).await;
    let elapsed = start.elapsed().as_micros() as u64;
    warn!(app.task_duration = elapsed, "task takes too long");
    "Hello, World!".to_string()
}
