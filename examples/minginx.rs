use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{
    io,
    net::{TcpListener, TcpStream},
};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    upstream_addr: String,
    listen_addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = resolve_config();
    // 这里使用 Arc 是因为 config 需要在多个异步任务中被共享和使用。
    // 每当一个新的连接被接受时，就会创建一个新的异步任务来处理这个连接。
    // 这个异步任务需要访问 config 来获取上游服务器的地址。
    let config = Arc::new(config);

    info!("Upstream is {}", config.upstream_addr);
    info!("Listening on {}", config.listen_addr);

    let listener = TcpListener::bind(&config.listen_addr).await?;
    loop {
        let (client, addr) = listener.accept().await?;
        info!("Accepted connection from {}", addr);
        // let cloned_config = config.clone();
        // 如果 config 是 Arc<T> 类型，推荐使用 Arc::clone(&config)，因为它效率更高且意图明确
        let cloned_config = Arc::clone(&config);
        tokio::spawn(async move {
            let upstream = TcpStream::connect(&cloned_config.upstream_addr).await?;
            proxy(client, upstream).await?;
            Ok::<(), anyhow::Error>(())
        });
    }

    #[allow(unreachable_code)]
    Ok::<(), anyhow::Error>(())
}

async fn proxy(mut client: TcpStream, mut upstream: TcpStream) -> Result<()> {
    // Splits a TcpStream into a read half and a write half, which can be used to read and write the stream concurrently.
    let (mut client_read, mut client_write) = client.split();
    let (mut upstream_read, mut upstream_write) = upstream.split();
    // io::copy 从 client_read 中读取数据并写入到 upstream_write
    let client_to_upstream = io::copy(&mut client_read, &mut upstream_write);
    // io::copy 从 upstream_read 中读取数据并写入到 client_write
    let upstream_to_client = io::copy(&mut upstream_read, &mut client_write);

    // 并发执行两个数据传输操作，并等待它们都完成。
    // try_join! 宏会在两个 Future 都完成时返回结果，如果任何一个 Future 返回错误，则立即返回错误。
    match tokio::try_join!(client_to_upstream, upstream_to_client) {
        Ok((n, m)) => info!(
            "proxied {} bytes from client to upstream, {} bytes from upstream to client",
            n, m
        ),
        Err(e) => warn!("error proxying: {:?}", e),
    }
    Ok(())
}

fn resolve_config() -> Config {
    Config {
        upstream_addr: "0.0.0.0:8080".to_string(),
        listen_addr: "0.0.0.0:8081".to_string(),
    }
}
