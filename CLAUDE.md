# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Open CoreUI is a lightweight, Rust-based reimplementation of Open WebUI v0.6.32. It provides a desktop client and backend server with significantly reduced memory footprint and hardware requirements compared to the original Python-based version.

## Architecture

This is a multi-component project with three main parts in a Cargo workspace:

### 1. Backend (`backend/`)
- **Language**: Rust
- **Framework**: Actix-web
- **Database**: SQLite (with Redis support for scaling)
- **Features**:
  - OpenAI-compatible API endpoints (`/openai/*`)
  - Native Rust Socket.IO implementation for real-time communication
  - Vector database integration for RAG capabilities
  - WebSocket streaming for chat (`/api/ws/chat`)
  - JWT-based authentication with configurable middleware
  - Embedded frontend support (optional)
  - Comprehensive health checks (`/health`, `/health/db`)

Key files:
- `backend/src/main.rs` - Main server entry point with initialization logic for all subsystems
- `backend/src/config.rs` - Comprehensive configuration management from environment variables
- `backend/src/routes/` - API route handlers (`/api/v1/*`, `/openai/*`)
- `backend/src/services/` - Business logic services (models, users, embeddings)
- `backend/src/socketio/` - Native Socket.IO implementation with Redis adapter
- `backend/src/models/` - Database models and schema definitions
- `backend/src/middleware.rs` - Authentication and security middleware

### 2. Frontend (`frontend/`)
- **Language**: TypeScript/Svelte
- **Framework**: SvelteKit with Vite
- **Styling**: TailwindCSS v4
- **Features**:
  - Rich text editor with TipTap extensions
  - Real-time chat with WebSocket and native Socket.IO client
  - Collaborative editing with Yjs CRDTs
  - Markdown rendering with Mermaid diagram support
  - File upload and image processing
  - Code syntax highlighting with CodeMirror
  - Audio TTS/STT capabilities (OpenAI, Azure)
  - Multi-provider image generation support

Key dependencies:
- SvelteKit for SSR/routing and progressive web app capabilities
- TipTap for rich text editing with collaborative extensions
- Yjs for collaborative editing with CRDTs
- Socket.IO client for real-time features
- TailwindCSS for responsive styling

### 3. Desktop App (`src-tauri/`)
- **Framework**: Tauri 2.x with workspace integration
- **Language**: Rust
- **Features**: Native desktop wrapper that launches the integrated backend server

## Development Commands

### Frontend Development
```bash
cd frontend
bun install          # Install dependencies (uses Bun package manager)
bun run dev          # Start development server (default port with --host)
bun run dev:5050     # Start on port 5050 specifically
bun run build        # Production build (outputs to build/)
bun run check        # Type checking with svelte-check
bun run test:frontend # Run tests with Vitest
bun run format       # Format code with Prettier
```

### Backend Development
```bash
cd backend
cargo build --release          # Production build with all features
cargo run                      # Development server
cargo run --no-default-features # Slim build without embedded frontend
cargo test                     # Run backend tests
```

### Desktop App Development
```bash
cargo tauri dev    # Development mode (builds and runs desktop app)
cargo tauri build   # Production build (creates distributable)
```

### Full Project Development (using Makefile)
```bash
make prepare-frontend    # Install frontend dependencies
make prepare-backend     # Fetch backend dependencies
make prepare-desktop     # Fetch Tauri dependencies
make build-frontend      # Build frontend (required for embedded backend)
make build-backend       # Build backend with embedded frontend
make build-backend-slim  # Build backend without frontend (requires external static files)
make run-backend         # Run backend server in development
make run-backend-slim    # Run slim backend without embedded frontend
make run-desktop         # Run desktop app in development
make build-desktop       # Build desktop app for production distribution
make update-version      # Interactive version update across all components
make build-arch-pkg      # Build Arch Linux package
```

## Environment Configuration

The backend is highly configurable through environment variables. See `CLI.md` for comprehensive documentation. Key variables include:

- `PORT` - Server port (default: 8168)
- `HOST` - Server host (default: 0.0.0.0)
- `OPENAI_API_KEY` - OpenAI API key
- `ENABLE_RAG` - Enable RAG features (default: true)
- `VECTOR_DB` - Vector database type (chroma)
- `REDIS_URL` - Redis connection URL for scaling
- `CONFIG_DIR` - Configuration directory (default: ~/.config/open-coreui)
- `ENABLE_RANDOM_PORT` - Use random OS-assigned port for dynamic allocation
- `RUST_LOG` - Rust logging level (default: info)

## Key Architectural Patterns

### Real-time Communication
- **Dual transport**: Both WebSocket (`/api/ws/chat`) and Socket.IO (`/socket.io`) are supported
- **Native Socket.IO**: Custom Rust implementation with Redis adapter for horizontal scaling, includes:
  - Yjs collaborative editing with CRDT synchronization
  - Rate limiting and backpressure handling
  - Connection recovery and presence tracking
  - Metrics and observability
- **State management**: Shared `AppState` contains database, config, and real-time managers

### Authentication & Authorization
- JWT-based authentication with configurable expiration (`JWT_EXPIRES_IN`)
- Middleware-based route protection (`middleware::AuthMiddleware`)
- Support for API keys, LDAP, and SCIM 2.0 integrations
- Role-based access control with configurable user roles

### API Design
- OpenAI-compatible endpoints at `/openai/*` for drop-in compatibility
- RESTful API at `/api/v1/*` for application-specific features
- Legacy endpoints maintained for backward compatibility
- Comprehensive health checks at `/health` and `/health/db`

### Database Design
- SQLite with connection pooling and automatic migrations
- Model abstraction in `backend/src/models/` with async `sqlx` integration
- Configuration persistence through `services::ConfigService`
- Supports external SQLite files or embedded database

### Frontend Architecture
- SvelteKit with TypeScript and Vite build system
- Component-based architecture with real-time state synchronization
- Progressive Web App capabilities with service worker support
- Rich text editing with collaborative extensions

### Vector Database & RAG
- Pluggable vector database support (Chroma, others via factory pattern)
- Multiple embedding providers: OpenAI, Knox Chat, local Candle-based transformers
- Configurable chunking, retrieval, and reranking strategies
- Graceful fallback when RAG features are unavailable

## Testing

- Frontend: `bun run test:frontend` (uses Vitest with test files in `frontend/src/tests/`)
- Backend: `cargo test` in the backend directory (unit and integration tests)
- Socket.IO health monitoring at `/api/socketio/health`

## Deployment

### Backend Server
1. Build with `make build-backend` for self-contained binary with embedded frontend
2. Use `make build-backend-slim` for smaller binary requiring external `STATIC_DIR`
3. Configure environment variables (see `CLI.md` for complete reference)
4. Run binary directly or use Docker/containerization

### Desktop Application
1. Build with `make build-desktop` for platform-specific distributables
2. Single executable with embedded backend server
3. No external dependencies required (self-contained deployment)

### Static File Serving
- Embedded frontend served by default via `rust-embed`
- External directory via `STATIC_DIR` environment variable
- Automatic fallback between embedded and external files

## Development Notes

### Socket.IO Implementation
- Native Rust Socket.IO implementation replaces Node.js dependency
- Redis adapter enables horizontal scaling across multiple server instances
- Comprehensive cleanup tasks prevent memory leaks in long-running processes
- Yjs document synchronization enables real-time collaborative editing

### Embedding System
- Fallback chain: Local sentence transformers → Knox Chat → OpenAI
- Optional feature gates (`embeddings` feature for local inference)
- Automatic model downloading from HuggingFace on first use
- Thread-safe embedding generation with connection pooling

### Performance Optimizations
- HTTP client connection pooling and keep-alive settings
- Async database operations with connection pooling
- Compression (gzip/brotli) and security headers
- Configurable timeouts and rate limiting

### Configuration Management
- Environment-based configuration with sensible defaults
- Runtime config updates through `MutableConfig` wrapper
- Database-stored configuration overrides
- Comprehensive validation and error handling

## Debugging

- Structured logging with `tracing` and configurable log levels
- Frontend hot reload and source maps in development
- Socket.IO connection monitoring at `/api/socketio/health`
- Database connectivity check at `/health/db`
- Request/response logging through actix-web Logger middleware

## File Structure Notes

- Frontend built to `frontend/build/` then embedded in backend binary
- External static files served from `{CONFIG_DIR}/build/` when available
- Database automatically created at `{CONFIG_DIR}/data.sqlite3`
- Configuration supports both environment variables and config files