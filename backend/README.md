# 🦀 Project Babel - Backend

**Framework:** Rust (Axum)
**Realtime:** Socketioxide (WebSocket)
**Database:** In-memory (DashMap)

## 🚀 Quick Start (Recommended)

The most recommended way to run the backend is via Docker Compose from the root directory:

```bash
docker-compose up -d backend
```

## 🛠️ Local Development

If you prefer to run it locally without Docker:

1. **Install Rust** (via rustup)
2. **Run Server**
   ```bash
   cargo run
   ```
3. **Run Tests**
   ```bash
   cargo test
   ```

## 📂 Structure

- `src/main.rs`: Entry point and server initialization.
- `src/server.rs`: API routes and WebSocket handlers.
- `src/room.rs`: Chat room logic and state management.
- `src/filter.rs`: Censorship filtering engine.
- `src/words.rs`: Word list generation and management.
- `src/data.rs`: Type definitions and constants.

## 📄 API Documentation

When the server is running, visit:
`http://localhost:3000/swagger-ui/` (if enabled)
