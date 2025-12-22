use quote_streaming::{Commands, StockQuote};
use rancor::Error;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, TcpStream, UdpSocket};
use std::sync::Arc;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:5152")?;
    println!("Connected to the server!");

    let command = Commands::Stream {
        ticker: vec!["AAPL".to_string(), "GOOGL".to_string()],
        address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        port: 5153,
    };
    let socket = Arc::new(UdpSocket::bind((
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        5153,
    ))?);

    let bytes = rkyv::to_bytes::<Error>(&command).unwrap();
    stream.write_all(bytes.as_slice())?;

    send_ping(Arc::clone(&socket));

    loop {
        let mut buffer = [0; 1024];
        let len = socket.recv(&mut buffer)?;
        let quote = rkyv::from_bytes::<StockQuote, rancor::Error>(&buffer[..len])?;
        println!("{}", quote);
    }

    Ok(())
}

fn send_ping(socket: Arc<UdpSocket>) {
    thread::spawn(move || {
        let ping = rkyv::to_bytes::<Error>(&quote_streaming::KeepAlive::Ping).unwrap();
        loop {
            socket.send_to(&ping, ("127.0.0.1", 5152)).unwrap();
            thread::sleep(std::time::Duration::from_secs(5));
        }
    });
}
