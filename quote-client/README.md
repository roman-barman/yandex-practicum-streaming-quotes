# quote-client

The client component of the Yandex Practicum Streaming Quotes project.

`quote-client` connects to the `quote-server` to request and display real-time stock quote streams.

## Features

- **Request Subscription**: Send requests to the server to stream quotes for a set of tickers.
- **UDP Listener**: Listens for incoming stock quote updates on a local UDP port.
- **Display**: Outputs received stock quotes to the console.
- **Command Line Interface**: Supports providing tickers via a file or directly as arguments.

## How it Works

1. The client connects to the server via TCP.
2. It sends a `StreamTickers` request, specifying which tickers it wants to follow and which local UDP port it will be listening on for updates.
3. The client then starts a UDP listener and waits for `StockQuote` data from the server.
4. As quotes arrive, they are formatted and printed to the standard output.

## Running the Client

To run the client, use `cargo run`. You must provide the server's address and port.

### Using a Ticker File

```bash
cargo run --bin quote-client -- -a 127.0.0.1 -p 5152 file --tickers-file my_tickers.txt
```

### Providing Tickers as Arguments

```bash
cargo run --bin quote-client -- -a 127.0.0.1 -p 5152 args -t AAPL -t MSFT -t GOOGL
```

### Options

- `-a, --server-address <ADDRESS>`: The IP address of the quote server.
- `-p, --server-port <PORT>`: The TCP port of the quote server.
- `-u, --udp-port <PORT>`: (Optional) The local UDP port to listen for quotes (default: 5153).

#### Subcommands

- `file`: Load tickers from a file.
    - `-f, --tickers-file <PATH>`: Path to the file.
- `args`: Provide tickers as arg-separated values.
    - `-t, --tickers <TICKER>...`: List of ticker symbols.
