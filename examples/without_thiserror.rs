use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::num::ParseIntError;

#[derive(Debug)]
pub enum MyError {
    Io(io::Error),
    Parse(ParseIntError),
}

// impl std::error::Error for MyError {}
// 打印错误信息， thiserror 的这个配置会实现一样的效果 #[error("File IO error: {0}")]
impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::Io(err) => write!(f, "File IO error: {}", err),
            MyError::Parse(err) => write!(f, "Parse error: {}", err),
        }
    }
}

// thiserror 会自动帮我们实现 From trait
impl From<io::Error> for MyError {
    fn from(err: io::Error) -> MyError {
        MyError::Io(err)
    }
}

impl From<ParseIntError> for MyError {
    fn from(err: ParseIntError) -> MyError {
        MyError::Parse(err)
    }
}

fn read_config() -> Result<String, MyError> {
    let mut file = File::open("config.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn parse_port(config: &str) -> Result<u16, MyError> {
    let port: u16 = config.trim().parse()?;
    Ok(port)
}

fn main() {
    match run() {
        Ok(_) => {
            println!("Ok")
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

fn run() -> Result<(), MyError> {
    let config = read_config()?;
    let port = parse_port(&config)?;
    println!("Config port: {}", port);
    Ok(())
}
