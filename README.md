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

## 宏

### derive_builder

使用 `derive_builder` 宏可以为结构体生成一个构造器。
比如下面的代码，我们定义了一个 `User` 结构体，然后使用 `derive_builder` 宏为 `User` 结构体生成一个 `UserBuilder` 结构体，`UserBuilder` 结构体包含了 `name` 和 `age` 两个字段的 setter 方法，以及一个 `build` 方法用于构建 `User` 结构体。

```rust
use anyhow::Result;
use derive_builder::Builder;

#[derive(Builder, Debug)]
struct User {
    name: String,
    age: u32,
}

fn main() -> Result<()> {
    let user = UserBuilder::default()
        .name("Alice".to_string())
        .age(30)
        .build()?;

    println!("{:?}", user);

    Ok(())
}
```

下面是一个更复杂的例子。

```rust
use anyhow::Result;
use chrono::{Datelike, DateTime, Utc};
use derive_builder::Builder;

#[derive(Builder, Debug, Default)]
#[builder(build_fn(name = "mybuild"))]
struct User {
    // 实现 from 方法，使得我们可以直接传递字符串字面量
    #[builder(setter(into))]
    name: String,
    age: u32,
    // strip_option 使你能够直接传递非 Option 类型的值，这样就不需要手动包装在 Some 中了
    // default 使得 email 字段在 build 时可以不传递，而使用默认值 None, 我们使用 #[derive(Default)] 为 User 结构体实现了 Default trait
    #[builder(setter(into, strip_option), default)]
    email: Option<String>,
    // 设置 country 字段的默认值为 "China"
    #[builder(setter(into), default = "String::from(\"China\")")]
    country: String,
    // 设置 height 字段的默认值为 180
    #[builder(default = "180")]
    height: u32,
    // skills 字段是一个 Vec<String> 类型，可以通过 skill 方法多次添加元素
    #[builder(default = "vec![]", setter(each(name="skill", into)))]
    skills: Vec<String>,
    #[builder(setter(custom))]
    dob: DateTime<Utc>,
    #[builder(setter(skip))]
    calculateAge: u32,
}

impl UserBuilder {
    // 根据 dob 字段的值计算年龄，并将其设置为 calculateAge 字段的值
    pub fn build(&self) -> Result<User> {
        let mut user = self.mybuild()?;
        user.calculateAge = (Utc::now().year() - user.dob.year()) as _;
        Ok(user)
    }
    // 受一个字符串参数 value，尝试将其解析为 RFC 3339 格式的日期时间，然后将其转换为 UTC 时间，并设置为 dob 字段的值。
    // 如果解析失败，dob 字段的值将被设置为 None。
    // Self 表示 UserBuilder 类型本身，&mut Self 表示一个可变引用。
    pub fn dob(&mut self, value: &str) -> &mut Self {
        self.dob = DateTime::parse_from_rfc3339(value)
            .map(|dt| dt.with_timezone(&Utc))
            // 如果解析失败（例如，value 不是一个有效的 RFC 3339 日期时间），
            // parse_from_rfc3339 方法将返回一个 Err，ok 方法将将其转换为 None。
            .ok();
        self
    }
}

fn main() -> Result<()> {
    let user = UserBuilder::default()
        .name("Alice")
        .age(30)
        .email("seven@example.com")
        .skill("programming")
        .skill("debugging")
        .dob("1990-01-01T00:00:00Z")
        .build()?;

    // User { name: "Alice", age: 30, email: Some("seven@example.com"), country: "China", height: 180, skills: ["programming", "debugging"], dob: 1990-01-01T00:00:00Z, calculate_age: 34 }
    println!("{:?}", user);

    Ok(())
}
```


### derive_more

`derive_more` 宏允许我们在结构体和枚举上轻松实现常见的 trait，例如 `Add`, `Sub`, `From`, `Display` 等等。

```rust
use derive_more::{Add, Constructor, Deref, DerefMut, Display, From, Sub};

// 实现 Add 和 Sub trait，允许我们对 Point 类型的实例进行加法和减法操作
#[derive(Add, Sub, Debug, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

#[allow(unused)]
// 实现 From trait，允许我们将 i32 类型的值转换为 Age 类型的值
#[derive(From, Debug)]
struct Age(i32);

// 实现 Display trait，允许我们自定义 MyEnum 类型的实例的显示方式
#[derive(Display)]
enum MyEnum {
    #[display(fmt = "int: {}", _0)]
    Int(i32),
    #[display(fmt = "nothing")]
    Nothing,
}

#[allow(unused)]
// 实现 Constructor trait，允许我们使用 new 方法创建 MyStruct 类型的实例
#[derive(Constructor, Debug)]
struct MyStruct {
    x: i32,
    y: i32,
}

// Deref：允许你通过 &T（不可变引用）访问内部数据。主要用于实现不可变解引用。
// DerefMut：允许你通过 &mut T（可变引用）访问内部数据。主要用于实现可变解引用。
#[derive(Deref, DerefMut, Debug)]
struct MyVec(Vec<i32>);

fn main() {
    let p1 = Point { x: 1, y: 2 };
    let p2 = Point { x: 3, y: 4 };

    let p3 = p1 + p2;
    let p4 = p3 - p1;

    println!("{:?}", p3); // Point { x: 4, y: 6 }
    println!("{:?}", p4); // Point { x: 3, y: 4 }

    let my_age: Age = 30.into();
    println!("{:?}", my_age); // Age(30)

    let myenum = MyEnum::Int(10);
    println!("{}", myenum); // int: 10
    let myenum2 = MyEnum::Nothing;
    println!("{}", myenum2); // nothing

    let p = MyStruct::new(1, 2);
    println!("{:?}", p); // Point { x: 1, y: 2 }

    let mut my_vec = MyVec(vec![1, 2, 3]);
    // 使用 Deref 实现自动解引用
    println!("Length: {}", my_vec.len());
    // 使用 DerefMut 实现可变解引用
    my_vec.push(4);
    // 验证元素是否成功添加
    println!("{:?}", my_vec);
}
```

### strum

`strum` 是一个流行的 derive 宏库，它可以自动为枚举类型派生出一些实用的 trait，方便我们对枚举进行各种转换和操作。比如：
- `EnumIter`：允许我们对枚举类型进行迭代。
- `EnumMessage`：允许我们为枚举类型添加一个自定义的消息。
- `VariantNames`：允许我们获取枚举类型的所有变体名称。
- `EnumCount`：允许我们获取枚举类型的变体数量。
- `Display`：允许我们为枚举类型实现 Display trait。

```rust
use std::string::ToString;
use strum::{
    Display, EnumCount, EnumIs, EnumIter, IntoEnumIterator, VariantNames,
};

#[allow(unused)]
#[derive(Display, EnumIs, EnumIter, VariantNames, Debug)]
enum Color {
    #[strum(serialize = "redred")]
    Red,
    Green {
        range: usize
    },
    Blue(usize),
    Yellow,
    #[strum(to_string = "purple with {sat} saturation")]
    Purple {
        sat: usize
    },
}

#[derive(Debug, EnumCount, EnumIter)]
enum Week {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

fn main() {
    // Display
    // uses the serialize string for Display
    let red = Color::Red;
    assert_eq!(String::from("redred"), format!("{}", red));
    // by default the variants Name
    let yellow = Color::Yellow;
    assert_eq!(String::from("Yellow"), yellow.to_string());
    // or for string formatting
    println!(
        "blue: {}, green: {}",
        Color::Blue(10),
        Color::Green { range: 42 }
    );
    // you can also use named fields in message
    let purple = Color::Purple { sat: 10 };
    assert_eq!(String::from("purple with 10 saturation"), purple.to_string());

    // EnumCount
    assert_eq!(7, Week::COUNT);
    assert_eq!(Week::iter().count(), Week::COUNT);

    // EnumIs
    assert!(Color::Red.is_red());
    assert!(Color::Green { range: 42 }.is_green());

    // VariantNames
    println!("{:?}", Color::VARIANTS);

    // EnumIter
    for color in Color::iter() {
        println!("My favorite color is {:?}", color);
    }
}
```

## Tokio

### Tokio 基本使用

以下是一段最简单的 Tokio 代码，使用 #[tokio::main] 宏来启动一个异步的 main 函数。

```rust
#[tokio::main]
async fn main() {
    let a = 10;
    let b = 20;
    println!("{} + {} = {}", a, b, a + b);
}
```

使用 `cargo expand` 命令可以展开宏。

```bash
cargo expand --example tokio0
```

展开后的代码如下，这里重点关注 tokio 运行时初始化和执行异步任务：
- 使用 `tokio::runtime::Builder::new_multi_thread()` 创建一个多线程运行时构建器。
- 调用 `enable_all()` 方法启用所有 Tokio 特性（如定时器、IO 等）。
- 调用 `build()` 方法构建运行时，并使用 `expect` 来处理可能的构建错误。
- 最后，使用 `block_on(body)` 来运行异步任务 `body`，并等待其完成。`block_on` 会阻塞当前线程，直到 `body` 完成。这意味着在 `block_on` 返回之前，主线程将不会继续执行。这是从异步上下文切换回同步上下文的一种方式。

```rust
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
fn main() {
    let body = async {
        let a = 10;
        let b = 20;
        {
            ::std::io::_print(format_args!("{0} + {1} = {2}\n", a, b, a + b));
        };
    };
    #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
    // tokio 运行时初始化和执行异步任务：
    {
        return tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
```

当我们明白了 `#[tokio::main]` 宏的工作原理后，我们可以手动创建一个 Tokio 运行时，然后使用 `tokio::spawn` 函数来执行异步任务。
- 使用 `tokio::runtime::Builder` 创建了一个新的 Tokio 运行时，并配置为当前线程运行时。
- 使用 `block_on` 在新创建的运行时上运行 `run` 函数，这会阻塞当前线程直到 `run` 函数完成。
- 在 `run` 函数在运行时上启动了两个异步任务：
  - -第一个任务读取 Cargo.toml 文件并打印文件长度。
  - 第二个任务执行耗时的阻塞任务并打印结果。

```rust
use std::{thread, time::Duration};

use tokio::{
    fs,
    runtime::{Builder, Runtime},
    time::sleep,
};

fn expensive_blocking_task(s: String) -> String {
    thread::sleep(Duration::from_millis(800));
    blake3::hash(s.as_bytes()).to_string()
}

fn main() {
    let handle = thread::spawn(|| {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(run(&rt));
    });

    handle.join().unwrap();
}


async fn run(rt: &Runtime) {
    rt.spawn(async {
        println!("future 1");
        let content = fs::read("Cargo.toml").await.unwrap();
        println!("content: {:?}", content.len());
    });

    rt.spawn(async {
        println!("future 2");
        let result = expensive_blocking_task("hello".to_string());
        println!("result: {}", result);
    });

    sleep(Duration::from_secs(1)).await;
}
```

我们也可以使用 `#[tokio::main]` 来简化运行时的创建和管理，不需要手动调用 `block_on` 来运行异步任务。

```rust
use std::{thread, time::Duration};

use tokio::{
    fs,
    // time::sleep,
};

fn expensive_blocking_task(s: String) -> String {
    thread::sleep(Duration::from_millis(8000));
    blake3::hash(s.as_bytes()).to_string()
}

// 使用 #[tokio::main] 宏将 main 函数标记为异步函数，并自动创建 Tokio 运行时。
#[tokio::main]
async fn main() {
    let handler1 = tokio::spawn(async {
        println!("future 1");
        let content = fs::read("Cargo.toml").await.unwrap();
        println!("content: {:?}", content.len());
    });

    let handler2 = tokio::spawn(async {
        println!("future 2");
        let result = expensive_blocking_task("hello".to_string());
        println!("result: {}", result);
    });

    // sleep 的时候主线程会暂时让出控制权，而运行时中的其他线程会继续执行异步任务。
    // 因此，即使主线程的 sleep 时间很短，异步任务仍然可以在后台完成执行。
    // sleep(Duration::from_millis(1)).await;

    // 但是更好的方式是使用 tokio::join! 来等待所有的异步任务完成。
    let (res1, res2) = tokio::join!(handler1, handler2);

    if let Err(e) = res1 {
        println!("Error in handle1: {:?}", e);
    }
    if let Err(e) = res2 {
        println!("Error in handle2: {:?}", e);
    }
}
```

### 使用 Tokio 编程简单的反向代理

主要代码如下：

```rust
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
```


首先启动之前写的一个 HTTP 程序作为 upstream，监听在 8080 端口。

```bash
cargo run --example axum_tracing
```

然后启动反向代理，监听在 8081 端口，将请求原样转发给 upstream。

```bash
cargo run --example minginx
```

客户端请求反向代理。

```bash
curl http://localhost:8081

# 响应内容
Hello, World!%
```

### 使用 Tokio 开发一个聊天室程序

定义以下 3 个数据结构：

```rust

// 用存储和管理所有连接到服务器的客户端，每个客户端地址映射到一个消息发送通道。
// mpsc::Sender：mpsc（多生产者，单消费者）通道允许从多个生产者发送消息到一个消费者，例如当有新的客户端连接或者离开时，向所有客户端广播这条消息。
// 使用 Arc<Message> 是为了在多个任务之间高效地共享消息，而不需要复制消息的内容。
#[derive(Debug, Default)]
struct State {
    peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>
}

// 表示单个客户端连接，包含用户名和用于处理消息流的读取部分。
// username 表示客户端的用户名。
// SplitStream<Framed<TcpStream, LinesCodec>> 类型，表示客户端的网络流，它被分割成了读取和写入两部分，可以用于并发地读取和写入数据。
#[derive(Debug)]
struct Peer {
    username: String,
    stream: SplitStream<Framed<TcpStream, LinesCodec>>
}

// 表示不同类型的消息，包含用户加入、用户离开和聊天消息。
#[derive(Debug)]
enum Message {
    UserJoined(String),
    UserLeft(String),
    Chat {
        sender: String,
        content: String,
    }
}
```

处理连接到聊天服务器的客户端：
- 首先，接收客户端输入的用户名
- 接下来，调用 `state.add` 方法，将客户端添加到服务器的状态中。这个方法会返回一个 Peer 实例，表示新添加的客户端。
- 接着，创建一个 `UserJoined` 类型的消息，表示用户已经加入聊天，然后调用 `state.broadcast` 方法，将这条消息广播给所有其他的客户端。
- 然后，进入一个循环，不断地从流中读取客户端发送的消息。对于每一条消息，函数都会创建一个 `Chat` 类型的消息，然后将这条消息广播给所有其他的客户端。如果读取消息失败，或者流已经结束，函数会跳出循环。
- 最后，函数从服务器的状态中移除这个客户端，然后创建一个 `UserLeft` 类型的消息，表示用户已经离开聊天，然后将这条消息广播给所有其他的客户端。

```rust
async fn handle_client(state: Arc<State>, addr: SocketAddr, stream: TcpStream) -> Result<()> {
    // Framed 是一个封装，它将底层的 I/O 流（如 TcpStream）与一个编码器/解码器（Codec）组合在一起，提供了一个异步的、分块处理的流接口。这使得我们能够以更高层次的抽象来处理数据，而不必关心底层的字节操作。
    // LinesCodec 是 tokio_util::codec 提供的一个编码器/解码器，它专门用于处理基于行的文本协议。它能够将字节流解析为一行一行的文本，或者将文本编码为字节流。
    let mut stream = Framed::new(stream, LinesCodec::new());
    stream.send("Enter your username:").await?;

    let username = match stream.next().await {
        Some(Ok(username)) => username,
        Some(Err(e)) => return Err(e.into()),
        None => return Ok(()),
    };

    let mut peer = state.add(addr, username, stream).await;

    let message = Arc::new(Message::user_joined(&peer.username));
    info!("{}", message);
    state.broadcast(addr, message).await;

    while let Some(line) = peer.stream.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("Failed to read line from {}: {}", addr, e);
                break;
            }
        };

        let message = Arc::new(Message::chat(&peer.username, line));

        state.broadcast(addr, message).await;
    }

    // when while loop exit, peer has left the chat or line reading failed
    // remove peer from state
    state.peers.remove(&addr);

    // notify others that a user has left
    let message = Arc::new(Message::user_left(&peer.username));
    info!("{}", message);

    state.broadcast(addr, message).await;

    Ok(())
}
```

启动聊天室服务器：

```bash
cargo run --example chat
```

使用 telnet 命令连接服务器，并尝试发送消息：

![](https://chengzw258.oss-cn-beijing.aliyuncs.com/Article/20240607111702.png)
