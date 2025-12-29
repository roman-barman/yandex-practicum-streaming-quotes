# quote-server

The server component of the Yandex Practicum Streaming Quotes project.

`quote-server` is responsible for receiving requests from clients to stream stock quotes for specific tickers and delivering those quotes in real-time.

## Features

- **TCP Request Handling**: Listens for incoming client requests via TCP.
- **UDP Quote Streaming**: Streams stock quotes to clients using UDP for low-latency delivery.
- **Configurable Tickers**: Load a list of available tickers from a text file.
- **Logging**: Integrated tracing for monitoring and debugging.

## How it Works

1. The server starts and reads the available tickers from the specified file.
2. It listens for TCP connections on a configured address and port.
3. When a client sends a `StreamTickers` request, the server starts generating random quotes for those tickers and sends them to the client's specified UDP address and port.
4. The server also handles `Ping` requests to confirm its availability.

## Running the Server

To run the server, use `cargo run`:

```bash
cargo run --bin quote-server -- --tickers-file tickers.txt --port 5152
```

### Options

- `-t, --tickers-file <PATH>`: Path to the file containing ticker symbols (one per line).
- `-p, --port <PORT>`: (Optional) The port to listen on for TCP connections (default: 5152).
- `-a, --address <ADDRESS>`: (Optional) The address to listen on (default: 127.0.0.1).
- `-l, --log-level <LEVEL>`: (Optional) Set the logging level (Trace, Debug, Info, Warn, Error).
