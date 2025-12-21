use quote_streaming::Commands;
use rancor::Error;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, TcpStream};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:5152")?;
    println!("Connected to the server!");

    let command = Commands::Stream {
        ticker: vec!["AAPL".to_string(), "GOOGL".to_string()],
        address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    };
    let bytes = rkyv::to_bytes::<Error>(&command).unwrap();
    stream.write_all(bytes.as_slice())?;

    Ok(())
}
