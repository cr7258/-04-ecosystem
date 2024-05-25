# 召唤元素：Rust 生态系统概览

## thiserror 和 anyhow

在 Rust 中，thiserror 和 anyhow 是用于错误处理的两个常用库，它们各有用途和适用场景。

### thiserror

thiserror 允许你定义自己的错误类型，并为每个错误变体提供自定义的错误信息。你需要使用 `#[derive(Error)]` 来为你的错误类型派生 `Error` trait，并使用 `#[error("...")]` 属性来指定每个错误变体的错误信息。

thiserror 会为你的自定义错误类型自动实现 `From` trait。当你使用 `#[from]` 属性时，thiserror 会为相应的错误类型生成 `From` 实现。

通过为你的自定义错误类型实现 `From` trait，你可以将其他错误类型轻松地转换为你的错误类型。这在错误传播和处理过程中非常有用。例如，如果你的函数返回一个 `Result<T, MyError>`，而某个内部函数返回一个 `Result<T, OtherError>`，你可以使用 `?` 运算符将 `OtherError` 自动转换为 `MyError`，前提是 `MyError` 实现了 `From<OtherError>`，这允许你将底层的错误类型包装在你的自定义错误类型中。

### anyhow

anyhow 提供了一个 `anyhow::Error` 类型，它可以包装任何实现了 `std::error::Error` 特征的错误类型。你不需要定义自己的错误类型，而是直接使用 `anyhow::Error`。

### 如何选择 thiserror 和 anyhow

如果你想要设计自己的错误类型，同时给调用者提供具体的信息时，就使用 thiserror，例如当你在开发一个三方库代码时。如果你只想要简单，就使用 anyhow，例如在自己的应用服务中。

### 验证效果

结合使用了 anyhow 和 thiserror。

```bash
cargo run --example err

# 输出
size of anyhow::Error is 8
size of std::io::Error is 8
size of std::num::ParseIntError is 1
size of serde_json::Error is 8
size of string is 24
size of MyError is 24

Error: Can not find file: non-existen-file.txt

Caused by:
    No such file or directory (os error 2)
```

## 日志

### 主要代码

```rust
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
```

`#[instrument]` 用于自动为函数添加跟踪span。
```rust
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
```

### 验证效果

发起一次 HTTP 请求，控制台日志如下：
```bash
cargo run --example axum_tracing

# 输出
  2024-05-25T02:39:05.792497Z  INFO axum_tracing: Starting server on 0.0.0.0:8080
    at examples/axum_tracing.rs:40

  2024-05-25T02:39:08.884575Z DEBUG axum_tracing: index handler started
    at examples/axum_tracing.rs:47
    in axum_tracing::index_handler

  2024-05-25T02:39:09.010202Z  WARN axum_tracing: task takes too long, app.task_duration: 114268
    at examples/axum_tracing.rs:59
    in axum_tracing::long_task
    in axum_tracing::index_handler

  2024-05-25T02:39:09.010357Z  INFO axum_tracing: close, time.busy: 233µs, time.idle: 114ms
    at examples/axum_tracing.rs:54
    in axum_tracing::long_task
    in axum_tracing::index_handler

  2024-05-25T02:39:09.010438Z  INFO axum_tracing: index handler completed, http.status: 200
    at examples/axum_tracing.rs:50
    in axum_tracing::index_handler

  2024-05-25T02:39:09.010480Z  INFO axum_tracing: close, time.busy: 486µs, time.idle: 125ms
    at examples/axum_tracing.rs:45
    in axum_tracing::index_handler
```

由于我们设置在文件中只保留 INFO 级别以上的日志，所以文件日志如下：

```bash
tail -f /tmp/logs/ecosystem.log.2024-05-25

# 输出
  2024-05-25T02:39:05.792627Z  INFO axum_tracing: Starting server on 0.0.0.0:8080
    at examples/axum_tracing.rs:40

  2024-05-25T02:39:09.010294Z  WARN axum_tracing: task takes too long, app.task_duration: 114268
    at examples/axum_tracing.rs:59
    in axum_tracing::long_task
    in axum_tracing::index_handler

  2024-05-25T02:39:09.010451Z  INFO axum_tracing: index handler completed, http.status: 200
    at examples/axum_tracing.rs:50
    in axum_tracing::index_handler
```

## OpenTelemetry

### 启动 Jaeger

```bash
docker run -d -p16686:16686 -p4317:4317 -e COLLECTOR_OTLP_ENABLED=true jaegertracing/all-in-one:latest
```

浏览器输入 http://localhost:16686/ 可以查看 Jaeger 的 Web 界面。

![](https://chengzw258.oss-cn-beijing.aliyuncs.com/Article/20240525111822.png)

### 主要代码

初始化 Tracer 对象，设置了 exporter 的连接信息，以及 trace 的配置信息：例如 service.name 来标识服务名称，最大事件数，最大属性数等。

```rust
fn init_tracer() -> anyhow::Result<Tracer> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
            trace::config()
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(32)
                .with_max_attributes_per_span(64)
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    "axum-tracing",
                )])),
        )
        .install_batch(runtime::Tokio)?;
    Ok(tracer)
}
```

创建并注册 opentelemetry layer。

```rust
// opentelemetry tracing layer for tracing-subscriber
let tracer = init_tracer()?;
let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

// 注册 opentelemetry layer
tracing_subscriber::registry()
     .....
    .with(opentelemetry)
    .init();
```

### 验证效果

启动服务，curl 命令请求 http://localhost:8080/， 在 Jaeger 的 Web 界面可以看到如下的 trace 信息：

```
cargo run --example opentelemetry-tracingq
```

![](https://chengzw258.oss-cn-beijing.aliyuncs.com/Article/20240525113449.png)
