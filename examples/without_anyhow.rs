use std::fs::File;
use std::io::{self, Read};
use std::num::ParseIntError;

#[derive(Debug)]
pub enum MyError {
    Io(io::Error),
    Parse(ParseIntError),
}

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

fn main() -> Result<(), MyError> {
    let config = read_config()?;
    let port = parse_port(&config)?;
    println!("Config port: {}", port);
    Ok(())
}
