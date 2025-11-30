# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Open CoreUI is a lightweight, Rust-based reimplementation of Open WebUI v0.6.32. It provides a desktop client and backend server with significantly reduced memory footprint and hardware requirements compared to the original Python-based version.

## Architecture

This is a multi-component project with three main parts:

### 1. Backend (`backend/`)
- **Language**: Rust
- **Framework**: Actix-web
- **Database**: SQLite (with Redis support for scaling)
- **Features**:
  - OpenAI-compatible API endpoints
  - Native Rust Socket.IO implementation for real-time communication
  - Vector database integration for RAG capabilities
  - WebSocket streaming for chat
  - Authentication with JWT tokens
  - Embedded frontend support

Key files:
- `backend/src/main.rs` - Main server entry point with comprehensive initialization
- `backend/src/config.rs` - Configuration management from environment variables
- `backend/src/routes/` - API route handlers
- `backend/src/services/` - Business logic services
- `backend/src/socketio/` - Native Socket.IO implementation

### 2. Frontend (`frontend/`)
- **Language**: TypeScript/Svelte
- **Framework**: SvelteKit
- **Build Tool**: Vite
- **Styling**: TailwindCSS
- **Features**:
  - Rich text editor with TipTap
  - Real-time chat with WebSocket and Socket.IO
  - Collaborative editing with Yjs
  - Markdown rendering with Mermaid diagrams
  - File upload and image handling
  - Code syntax highlighting with CodeMirror
  - Audio TTS/STT capabilities

Key dependencies:
- SvelteKit for SSR/routing
- TipTap for rich text editing
- Yjs for collaborative editing
- Socket.IO client for real-time features
- TailwindCSS for styling

### 3. Desktop App (`src-tauri/`)
- **Framework**: Tauri 2.x
- **Language**: Rust
- **Features**: Native desktop wrapper that launches the backend server

## Development Commands

### Frontend Development
```bash
cd frontend
bun install          # Install dependencies
bun run dev          # Start development server (default port)
bun run dev:5050     # Start on port 5050
bun run build        # Production build
bun run check        # Type checking
bun run test:frontend # Run tests
bun run format       # Format code with Prettier
```

### Backend Development
```bash
cd backend
cargo build --release          # Production build
cargo run                      # Development server
cargo run --no-default-features # Slim build without embedded frontend
```

### Desktop App Development
```bash
cargo tauri dev    # Development mode
cargo tauri build   # Production build
```

### Full Project Development (using Makefile)
```bash
make prepare-frontend    # Install frontend dependencies
make prepare-backend     # Fetch backend dependencies
make build-frontend      # Build frontend
make build-backend       # Build backend with embedded frontend
make build-backend-slim  # Build backend without frontend
make run-backend         # Run backend server
make run-desktop         # Run desktop app in development
make build-desktop       # Build desktop app for production
```

## Environment Configuration

The backend is highly configurable through environment variables. See `CLI.md` for comprehensive documentation. Key variables include:

- `PORT` - Server port (default: 8168)
- `HOST` - Server host (default: 0.0.0.0)
- `OPENAI_API_KEY` - OpenAI API key
- `ENABLE_RAG` - Enable RAG features
- `VECTOR_DB` - Vector database type (chroma)
- `REDIS_URL` - Redis connection URL for scaling

## Key Architectural Patterns

### Real-time Communication
- **Dual transport**: Both WebSocket (`/api/ws/chat`) and Socket.IO (`/socket.io`) are supported
- **Native Socket.IO**: Custom Rust implementation with Redis adapter for horizontal scaling
- **Yjs Integration**: Collaborative document editing with CRDTs

### Authentication
- JWT-based authentication with configurable expiration
- Middleware-based route protection (`middleware::AuthMiddleware`)
- Support for API keys and OAuth integrations

### API Design
- OpenAI-compatible endpoints at `/openai/*` for easy integration
- RESTful API at `/api/v1/*` for application-specific features
- Legacy endpoints maintained for compatibility

### Database Design
- SQLite with connection pooling
- Migration system in `backend/src/schema.sql`
- Model abstraction in `backend/src/models/`

### Frontend Architecture
- SvelteKit with TypeScript
- Component-based architecture with shared state management
- Real-time features using Socket.IO client
- Progressive Web App capabilities

## Testing

- Frontend: `bun run test:frontend` (uses Vitest)
- Backend: Use `cargo test` in the backend directory
- Integration testing through the Makefile targets

## Deployment

### Backend Server
1. Build with `make build-backend` for a self-contained binary
2. Use `make build-backend-slim` for a smaller binary without embedded frontend
3. Configure environment variables (see `CLI.md`)
4. Run the binary directly

### Desktop Application
1. Build with `make build-desktop`
2. Distributes as a single executable with embedded backend
3. No external dependencies required

## Development Notes

- The backend embedding system supports multiple providers: OpenAI, Knox Chat, and local sentence transformers
- Vector database integration is optional and falls back gracefully
- Socket.IO includes comprehensive features: presence, rate limiting, connection recovery
- Frontend uses modern web APIs and requires Node.js >=18.13.0
- The project maintains compatibility with Open WebUI's API surface

## Debugging

- Backend logs include detailed tracing with structured output
- Frontend development includes hot reload and source maps
- Socket.IO connections can be monitored through `/api/socketio/health`
- Database health check available at `/health/db`

## File Structure Notes

- Frontend static files are embedded in the backend binary by default
- External static directory can be used via `STATIC_DIR` environment variable
- Configuration files are in TOML/YAML format
- Database migrations are automatic on startup