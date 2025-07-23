# Print My Bridge

<div align="center">
  <img src="src-tauri/icons/icon.png" alt="Print My Bridge Logo" width="128" height="128">
  <h3>A secure bridge application for remote printing</h3>
  <p>Built with Tauri, Rust, and modern web technologies</p>
</div>

## 🚀 Overview

Print My Bridge is a desktop application that creates a secure HTTP bridge for remote printing. It allows you to send print jobs to local printers through a REST API with authentication and rate limiting.

### ✨ Features

- 🖨️ **Remote Printing**: Send print jobs to local printers via HTTP API
- 🔐 **Secure Authentication**: Token-based API authentication
- 🚦 **Rate Limiting**: Configurable request rate limiting
- 🎯 **Cross-Platform**: Available for Windows, macOS, and Linux
- ⚙️ **Configurable**: Flexible configuration options
- 🔄 **Auto-Start**: Optional system startup integration
- 📱 **System Tray**: Minimize to system tray support
- 🌐 **CORS Support**: Configurable cross-origin resource sharing

## 📋 Requirements

- **Operating System**: Windows 10+, macOS 10.13+, or Linux
- **Rust**: 1.70+ (for development)
- **Node.js**: 16+ (for development)

## 🔧 Installation

### Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/your-username/print-my-bridge/releases) page:

- **Windows**: `Print My Bridge_x.x.x_x64_en-US.msi`
- **macOS**: `Print My Bridge.app` (Universal binary)
- **Linux**: `print-my-bridge_x.x.x_amd64.deb`

### Build from Source

1. **Clone the repository**:
   ```bash
   git clone https://github.com/your-username/print-my-bridge.git
   cd print-my-bridge
   ```

2. **Install Rust and Tauri CLI**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   cargo install tauri-cli
   ```

3. **Build the application**:
   ```bash
   cd src-tauri
   cargo tauri build
   ```

## ⚙️ Configuration

The application uses a TOML configuration file located at:

- **Windows**: `%APPDATA%/print-my-bridge/print-my-bridge.toml`
- **macOS**: `~/Library/Application Support/print-my-bridge/print-my-bridge.toml`
- **Linux**: `~/.config/print-my-bridge/print-my-bridge.toml`

### Configuration Options

```toml
# Server configuration
host = "127.0.0.1"
port = 8765
max_file_size_mb = 10
rate_limit_per_minute = 60

# Application settings
auto_start = false
minimize_to_tray = true

# Security settings
allowed_origins = ["*"]
allowed_file_types = [".pdf", ".txt", ".doc", ".docx"]
default_printer = ""
```

## 🔑 API Authentication

1. **Generate a token** through the application UI
2. **Include the token** in your API requests:
   ```bash
   curl -H "Authorization: Bearer YOUR_TOKEN" \
        -F "file=@document.pdf" \
        http://localhost:8765/api/print
   ```

## 📡 API Endpoints

### Health Check
```http
GET /health
```

**Response**:
```json
{
  "status": "ok",
  "service": "print-my-bridge",
  "version": "0.1.0"
}
```

### List Printers
```http
GET /api/printers
Authorization: Bearer YOUR_TOKEN
```

**Response**:
```json
{
  "printers": [
    {
      "name": "HP LaserJet Pro",
      "is_default": true,
      "status": "ready"
    }
  ]
}
```

### Print Document
```http
POST /api/print
Authorization: Bearer YOUR_TOKEN
Content-Type: multipart/form-data
```

**Parameters**:
- `file`: Document file (required)
- `printer`: Printer name (optional, uses default if not specified)
- `copies`: Number of copies (optional, default: 1)

**Example**:
```bash
curl -X POST \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -F "file=@document.pdf" \
  -F "printer=HP LaserJet Pro" \
  -F "copies=2" \
  http://localhost:8765/api/print
```

**Response**:
```json
{
  "success": true,
  "message": "Document sent to printer successfully",
  "job_id": "12345"
}
```

## 🛠️ Development

### Project Structure

print-my-bridge/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── api/         # HTTP API routes
│   │   ├── config/      # Configuration management
│   │   ├── gui/         # Tauri commands
│   │   ├── printer/     # Printer integration
│   │   └── main.rs      # Application entry point
│   └── tauri.conf.json  # Tauri configuration
├── ui/                  # Frontend UI
│   ├── index.html
│   ├── script.js
│   └── style.css
└── README.md


### Development Setup

1. **Install dependencies**:
   ```bash
   cd src-tauri
   cargo build
   ```

2. **Run in development mode**:
   ```bash
   cargo tauri dev
   ```

3. **Run tests**:
   ```bash
   cargo test
   ```

### Building for Different Platforms

#### macOS
```bash
# Intel (x86_64)
cargo tauri build --target x86_64-apple-darwin

# Apple Silicon (M1/M2)
cargo tauri build --target aarch64-apple-darwin

# Universal binary
cargo tauri build --target universal-apple-darwin
```

#### Windows
```bash
rustup target add x86_64-pc-windows-msvc
cargo tauri build --target x86_64-pc-windows-msvc
```

#### Linux
```bash
rustup target add x86_64-unknown-linux-gnu
cargo tauri build --target x86_64-unknown-linux-gnu
```

#### Build Script

Use the included build script for all platforms:
```bash
./build-all.sh
```

## 🔒 Security

- **Token Authentication**: All API endpoints (except `/health`) require a valid bearer token
- **Rate Limiting**: Configurable request rate limiting to prevent abuse
- **CORS Protection**: Configurable allowed origins
- **File Type Validation**: Restrict allowed file types for printing
- **Local Network Only**: Server binds to localhost by default

## 🐛 Troubleshooting

### Common Issues

#### Bridge Not Running
- Check if the application is running in the system tray
- Verify the configuration file exists and is valid
- Check if the configured port is available

#### Connection Refused
- Ensure the application is running
- Verify the correct host and port in configuration
- Check firewall settings

#### Print Job Fails
- Verify the printer is connected and online
- Check if the file type is allowed in configuration
- Ensure the file size doesn't exceed the limit

#### CORS Issues
- Update `allowed_origins` in configuration
- Use specific origins instead of `*` for production

### Debug Mode

Run the application from terminal to see detailed logs:
```bash
cd src-tauri
cargo run
```

### Logs

Application logs can be found at:
- **Windows**: `%APPDATA%/print-my-bridge/logs/`
- **macOS**: `~/Library/Logs/print-my-bridge/`
- **Linux**: `~/.local/share/print-my-bridge/logs/`

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

### Development Guidelines

- Follow Rust coding conventions
- Add tests for new features
- Update documentation as needed
- Ensure cross-platform compatibility

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Tauri](https://tauri.app/) - For the amazing desktop app framework
- [Warp](https://github.com/seanmonstar/warp) - For the HTTP server framework
- [Tokio](https://tokio.rs/) - For async runtime
- [Serde](https://serde.rs/) - For serialization

## 📞 Support

If you encounter any issues or have questions:

1. Check the [Issues](https://github.com/your-username/print-my-bridge/issues) page
2. Create a new issue with detailed information
3. Include logs and configuration details

## 🔄 Changelog

### v0.1.0 (Initial Release)
- Basic printing functionality
- Token-based authentication
- Rate limiting
- Cross-platform support
- System tray integration
- Configuration management

---