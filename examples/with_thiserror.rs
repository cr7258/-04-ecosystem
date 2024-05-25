use std::fs::File;
use std::io::Read;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("File IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] ParseIntError),
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

// fn main() -> Result<(), MyError> {
//     let config = read_config()?;
//     let port = parse_port(&config)?;
//     println!("Config port: {}", port);
//     Ok(())
// }

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
