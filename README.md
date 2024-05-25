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

### 示例代码

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
