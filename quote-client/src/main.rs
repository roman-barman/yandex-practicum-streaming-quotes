use quote_streaming::{Commands, StockQuote};
use rancor::Error;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, TcpStream, UdpSocket};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:5152")?;
    println!("Connected to the server!");

    let command = Commands::Stream {
        ticker: vec!["AAPL".to_string(), "GOOGL".to_string()],
        address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        port: 5153,
    };
    let socket = UdpSocket::bind((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5153))?;

    let bytes = rkyv::to_bytes::<Error>(&command).unwrap();
    stream.write_all(bytes.as_slice())?;

    loop {
        let mut buffer = [0; 1024];
        let len = socket.recv(&mut buffer)?;
        let quotes = rkyv::from_bytes::<Vec<StockQuote>, rancor::Error>(&buffer[..len])?;
        for quote in quotes {
            println!("{}", quote);
        }
    }

    Ok(())
}
