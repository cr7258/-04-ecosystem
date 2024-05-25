use std::fs::File;
use std::io::{self, Read};
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("File IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] ParseIntError),
}

pub fn read_config() -> Result<String, MyError> {
    let mut file = File::open("config.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn parse_port(config: &str) -> Result<u16, MyError> {
    let port: u16 = config.trim().parse()?;
    Ok(port)
}

// anyhow 会自动处理错误链并提供详细的错误信息。
// 输出结果：
// Error: File IO error: No such file or directory (os error 2)
//
// Caused by:
//     No such file or directory (os error 2)
fn main() -> anyhow::Result<()> {
    let config = read_config()?;
    let port = parse_port(&config)?;
    println!("Config port: {}", port);
    Ok(())
}
