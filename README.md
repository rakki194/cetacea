# Cetacea 🐋

A sleek, terminal-based Docker container monitoring tool with a beautiful TUI interface. Monitor your Docker containers in real-time with an intuitive, responsive interface.

## Features

- **Real-time Monitoring**: Live updates of container status every second
- **Beautiful TUI**: Clean, responsive terminal user interface using [ratatui](https://github.com/ratatui-org/ratatui)
- **Container Information**:
  - Container name and ID
  - Image details
  - Running state and health status
  - Port mappings
  - Creation time and uptime
  - Command information
- **Smart Layout**: Automatically adjusts to terminal size with responsive grid layout
- **Color Coding**:
  - Green: Running and healthy containers
  - Yellow: Running containers with uncertain health
  - Red: Stopped containers or unhealthy state
- **Cross-platform**: Supports both Linux (Unix socket) and Windows (HTTP) connections

## Installation

```bash
cargo install cetacea
```

## Usage

Simply run:

```bash
cetacea
```

### Controls

- `q`: Quit the application
- Terminal resize: UI automatically adjusts

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
