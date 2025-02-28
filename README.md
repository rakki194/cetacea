# Cetacea

A terminal UI Docker container monitor built with Rust.

## Features

- Real-time container monitoring
- Status indicators for running/stopped containers
- Health check status display
- Container details including:
  - Image name
  - Command
  - Creation time
  - Status
  - Port mappings
- Automatic refresh of container status
- Responsive grid layout that adapts to terminal size

## Usage

```bash
# Run with default settings (250ms refresh rate)
cetacea

# Run with custom refresh rate (e.g., 1 second)
cetacea -r 1000
```

### Options

- `-r, --refresh-rate <MS>`: Set the refresh rate in milliseconds (default: 250)
- `-h, --help`: Show help information
- `-V, --version`: Show version information

## Controls

- `q`: Quit the application

## Requirements

- Docker daemon running and accessible via Unix socket
- Rust 1.70 or later

## Building

```bash
cargo build --release
```

The binary will be available at `target/release/cetacea`.

## Development

### Prerequisites

- Rust (latest stable)
- Docker daemon running
- Cargo and standard Rust toolchain

### Building from Source

```bash
git clone https://github.com/rakki194/cetacea
cd cetacea
cargo build --release
```

### Running Tests

```bash
cargo test
```

Note: Some tests require a running Docker daemon to pass.

## Architecture

- **Docker Client**: Asynchronous communication with Docker daemon using hyper
- **TUI Layer**: Built with ratatui for responsive terminal rendering
- **Update Thread**: Background thread for real-time container status updates
- **Error Handling**: Comprehensive error types using thiserror

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [ratatui](https://github.com/ratatui-org/ratatui) for the amazing terminal user interface framework
- [tokio](https://tokio.rs/) for the robust async runtime
- [hyper](https://hyper.rs/) for HTTP client implementation
