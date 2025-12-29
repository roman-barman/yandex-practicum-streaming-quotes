# quote-streaming

A library crate for the Yandex Practicum Streaming Quotes project.

This crate provides the core data structures and serialization logic used by both the server and the client to communicate and handle stock quote data.

## Features

- **Data Structures**: Defines `StockQuote`, `Request`, and `Response`.
- **Serialization**: Implements efficient serialization and deserialization using the `rkyv` library.
- **Mock Generation**: Includes utility functions for generating random stock quotes for testing and demonstration.

## Core Components

- `StockQuote`: Represents a single stock quote update, including ticker, price, volume, and timestamp.
- `Request`: Enum for client-to-server messages (e.g., `StreamTickers`, `Ping`).
- `Response`: Enum for server-to-client messages (e.g., `Quote`, `Pong`, `Error`, `Ok`).

## Usage

This library is intended to be used as a dependency for the `quote-server` and `quote-client` crates within this workspace.
