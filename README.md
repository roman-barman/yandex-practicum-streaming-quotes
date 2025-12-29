# Yandex Practicum Streaming Quotes

A real-time stock quote streaming system built with Rust. This workspace implements a client-server architecture where the server generates and streams mock stock quotes to clients via UDP, while managing subscriptions over TCP.

## Workspace Structure

This project is organized into three main crates:

- **[quote-server](./quote-server)**: The server application that handles ticker subscriptions and streams quotes.
- **[quote-client](./quote-client)**: The client application that subscribes to tickers and displays the live stream.
- **[quote-streaming](./quote-streaming)**: A shared library containing common data structures and serialization logic.

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo

### Quick Start

1.  **Prepare a Tickers File**: Create a file named `tickers.txt` with some ticker symbols, one per line (e.g., AAPL, MSFT, TSLA).

2.  **Start the Server**:
    ```bash
    cargo run --bin quote-server -- --tickers-file tickers.txt
    ```

3.  **Start the Client**:
    Open a new terminal window and run:
    ```bash
    cargo run --bin quote-client -- -a 127.0.0.1 -p 5152 args -t AAPL -t MSFT
    ```

## Documentation

For more detailed information about each component, please refer to the individual README files in their respective directories:

- [quote-server/README.md](./quote-server/README.md)
- [quote-client/README.md](./quote-client/README.md)
- [quote-streaming/README.md](./quote-streaming/README.md)