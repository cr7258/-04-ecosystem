use std::fs::File;
use std::io::Read;

// fn read_config() -> anyhow::Result<String> {
//     let mut file = File::open("config.toml")?;
//     let mut contents = String::new();
//     file.read_to_string(&mut contents)?;
//     Ok(contents)
// }
//
// fn parse_port(config: &str) -> anyhow::Result<u16> {
//     let port: u16 = config.trim().parse()?;
//     Ok(port)
// }
//
// fn main() -> anyhow::Result<()> {
//     let config = read_config()?;
//     let port = parse_port(&config)?;
//     println!("Config port: {}", port);
//     Ok(())
// }

// 上面的写法的效果和下面的一样
// Result<T, anyhow::Error> 等效于 anyhow::Result<T>
fn read_config() -> Result<String, anyhow::Error> {
    let mut file = File::open("config.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn parse_port(config: &str) -> Result<u16, anyhow::Error> {
    let port: u16 = config.trim().parse()?;
    Ok(port)
}

fn main() -> Result<(), anyhow::Error> {
    let config = read_config()?;
    let port = parse_port(&config)?;
    println!("Config port: {}", port);
    Ok(())
}
