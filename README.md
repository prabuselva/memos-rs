# memos-rs

A Joplin-like note-taking application built with Rust and React. Provides a powerful, web-based note-taking experience with support for Markdown, user authentication, and import/export of notes from Tomboy and Gnote formats.

**Developed with:** [Qwen3-Coder-Next](https://huggingface.co/Qwen/Qwen3-Coder-Next) model using the [opencode](https://opencode.ai) framework.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.7+-blue.svg)](https://axum.rs/)
[![React](https://img.shields.io/badge/React-19+-blue.svg)](https://react.dev/)

## Demo

[![Watch the video]()](https://github.com/user-attachments/assets/353f5afd-3c29-45c1-811d-12b4992584df)

## 📝 Summary

memos-rs is a lightweight, self-hosted note-taking application that helps you organize your thoughts and ideas. Built with modern web technologies, it offers a clean interface, robust authentication, and flexible import/export capabilities for migrating from other note-taking apps.

## ✨ Features

- **Markdown Support** - Write notes in Markdown with live HTML preview
- **User Authentication** - Secure login with session management and password recovery
- **Multi-user Support** - Isolated notes per user with role-based access
- **Tomboy/Gnote Import** - Import notes from Tomboy and Gnote XML formats
- **Export to XML** - Export all notes as Tomboy XML
- **Search Functionality** - Search notes by title and content
- **Note Organization** - Tags, favorites, and archive support
- **Responsive UI** - Works on desktop and mobile devices
- **Dark Mode** - Built-in dark theme for comfortable reading

## 🖥️ System Requirements

- **Rust**: 1.75 or higher
- **Node.js**: 18 or higher (for frontend build)
- **RAM**: 512MB minimum, 1GB recommended
- **Disk Space**: 100MB for application + database
- **Database**: SQLite 3.35+ (embedded) or PostgreSQL 12+

## 🚀 Quick Start

### Using Pre-built Binaries

Download the appropriate binary for your platform from the [Releases](https://github.com/yourusername/memos-rs/releases) page:

- **Linux**: `memos-rs-linux-amd64.tar.gz`
- **Windows**: `memos-rs-windows-amd64.zip`

**Installation:**

```bash
# Linux
tar -xzf memos-rs-linux-amd64.tar.gz
./memos-rs

# Windows (PowerShell)
Expand-Archive memos-rs-*.zip
.\memos-rs.exe
```

The server will start on `http://localhost:3000`.

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/memos-rs.git
cd memos-rs

# Build the project
bash build.sh

# Start the server
./target/release/memos-rs
```

## 🛠️ Build Instructions

### Prerequisites

- Rust 1.75+ with Cargo
- Node.js 18+ with npm
- Git

### Build Process

1. **Install frontend dependencies and build**:

```bash
cd frontend
npm install
npm run build
cd ..
```

2. **Build Rust backend**:

```bash
# Release build
cargo build --release

# Or use the build script
bash build.sh
```

3. **Run the application**:

```bash
cargo run --release
```

By default, the server runs on `http://0.0.0.0:3000`.

### Build with Embedded Frontend

To create a single binary with the frontend embedded:

```bash
bash build.sh --embed-frontend
```

## ⚙️ Configuration

memos-rs uses a TOML configuration file. Create a `config.toml` file in the working directory:

```toml
[server]
host = "0.0.0.0"
port = 3000

[database]
kind = "SQLite"
path = ".memos-rs/data.sqlite"

[storage]
attachments_dir = ".memos-rs/attachments"

[auth]
session_duration_days = 7
password_reset_duration_hours = 1
max_login_attempts = 5
lockout_duration_minutes = 15
bcrypt_cost = 12
```

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `server.host` | Server host address | `0.0.0.0` |
| `server.port` | Server port | `3000` |
| `database.kind` | Database type (`SQLite` or `PostgreSQL`) | `SQLite` |
| `database.path` | Path to SQLite database file | `.memos-rs/data.sqlite` |
| `storage.attachments_dir` | Directory for attachments | `.memos-rs/attachments` |
| `auth.session_duration_days` | Session validity in days | `7` |
| `auth.password_reset_duration_hours` | Password reset token expiry | `1` |
| `auth.max_login_attempts` | Max login attempts before lockout | `5` |
| `auth.lockout_duration_minutes` | Lockout duration after failed attempts | `15` |
| `auth.bcrypt_cost` | Bcrypt hashing cost | `12` |

## 📡 API Documentation

Full API documentation is available in [docs/API.md](docs/API.md).

### Quick Reference

| Endpoint | Description |
|----------|-------------|
| `POST /api/register` | Register a new user |
| `POST /api/login` | Login and get session token |
| `GET /api/notes` | List all notes |
| `POST /api/notes` | Create a new note |
| `GET /api/notes/search?q={query}` | Search notes |
| `POST /api/import/tomboy` | Import Tomboy notes |
| `GET /api/export/tomboy` | Export all notes |
| `GET /health` | Health check |

## 📥 Import/Export

### Supported Import Formats

#### Tomboy XML

Import notes from Tomboy note-taking application:

```bash
curl -X POST http://localhost:3000/api/import/tomboy \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "notes": [
      "<?xml version=\"1.0\"?><note><title>My Note</title><content>Content</content></note>"
    ]
  }'
```

#### Gnote XML

Import notes from Gnote (GNOME Notes) application using the same import endpoints.

### Export

Export all notes as Tomboy XML:

```bash
curl -X GET http://localhost:3000/api/export/tomboy \
  -H "Authorization: Bearer YOUR_TOKEN"
```

## 📂 Project Structure

```
memos-rs/
├── docs/             # Documentation
│   └── API.md        # API documentation
├── src/
│   ├── api/          # API routes and handlers
│   ├── db/           # Database layer and queries
│   ├── models/       # Data models (Note, User, Session)
│   ├── services/     # Business logic (Auth, etc.)
│   ├── repositories/ # Data access layer
│   ├── import_export/ # Import/export functionality
│   ├── markdown.rs   # Markdown processing utilities
│   └── main.rs       # Application entry point
├── frontend/
│   ├── src/
│   │   ├── components/ # React components
│   │   ├── hooks/      # Custom React hooks
│   │   ├── lib/        # API clients and utilities
│   │   └── App.tsx     # Main React component
│   └── package.json
├── .github/
│   └── workflows/    # CI/CD workflows
├── Cargo.toml        # Rust dependencies
├── build.sh          # Build script
└── config.toml.example # Example configuration
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [memos](https://github.com/usememos/memos) - For inspiring the concept of open-source, self-hosted note-taking with Markdown-first approach.
- [Joplin](https://joplinapp.org/) for inspiration
- [Tomboy](https://github.com/gnome/tomboy) for XML format reference
- All the open-source libraries used in this project
- **Qwen3-Coder-Next** model and **opencode** framework for AI-assisted development

## 📞 Support

For support, please open an issue in the GitHub repository or contact the maintainers.

---

**Happy Note-Taking!** 📝
