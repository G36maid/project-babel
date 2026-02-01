# 🤖 AGENTS.md - Project Babel Guidelines

> **Context**: A multiplayer puzzle game with Vue 3 frontend and Rust (Axum) backend.  
> **Game**: Players use a symbolic language to communicate across censorship firewalls.

---

## 🏗️ Build & Run Commands

### Docker (Unified - Recommended)
**Working Directory**: Project root

| Action | Command | Description |
|--------|---------|-------------|
| Start All | `docker-compose up -d` | Start frontend + backend in detached mode |
| Stop All | `docker-compose down` | Stop all services |
| View Logs | `docker-compose logs -f` | Follow logs from all services |
| Rebuild | `docker-compose up -d --build` | Rebuild and restart after code changes |
| Frontend Only | `docker-compose up -d frontend` | Start only frontend service |
| Backend Only | `docker-compose up -d backend` | Start only backend service |

**Service URLs**:
- Frontend: `http://localhost:8080` (Nginx production build)
- Backend: `http://localhost:3000` (Rust Axum)

**Network**: Services communicate via `babel-network` bridge (defined in docker-compose.yml)

---

### Frontend (Vue 3 + TypeScript) - Local Dev
**Working Directory**: `frontend/`

| Action | Bun (Preferred) | npm (Fallback) | Docker Dev | Description |
|--------|---------------|-------------------|------------|-------------|
| Install | `bun install` | `npm install` | - | Install dependencies |
| Dev | `bun run dev` | `npm run dev` | `docker build --target development -t babel-dev . && docker run -p 5173:5173 -v $(pwd):/app babel-dev` | Vite dev server on `localhost:5173` |
| Build | `bun run build` | `npm run build` | `docker build --target production -t babel-prod .` | Production build |
| Preview | `bun run preview` | `npm run preview` | - | Preview production build |
| Type Check | `bunx vue-tsc -b` | `npx vue-tsc -b` | - | TypeScript compiler only |

**Lockfile**: `bun.lock` (text format, git-diffable)  
**Why Bun?** Project uses Bun as the primary tool because it is ~4-6× faster than npm. If `bun` is not available on your system, you can fallback to `npm`.

---

### Backend (Rust + Axum) - Local Dev
**Working Directory**: `backend/` or project root

| Action | Command | Docker Build | Description |
|--------|---------|--------------|-------------|
| Run | `cargo run` | `docker build -t babel-backend . && docker run -p 3000:3000 babel-backend` | Start server on `localhost:3000` |
| Build | `cargo build --release` | `docker build --target builder -t babel-backend-builder .` | Production build |
| Test | `cargo test` | - | Run all tests |
| Test Single | `cargo test test_name` | - | Run specific test |
| Check | `cargo check` | - | Fast compile check |

---

### Development Workflow Options

#### Option 1: Docker Compose (Recommended for consistency)
```bash
# Single command to start everything
docker-compose up -d

# View logs
docker-compose logs -f

# Stop everything
docker-compose down
```

#### Option 2: Local Development (Faster iteration)
```bash
# Terminal 1: Start backend
cd backend && cargo run

# Terminal 2: Start frontend (use Bun for speed)
cd frontend && bun run dev
```

#### Option 3: Hybrid (Backend Docker + Frontend Local)
```bash
# Terminal 1: Backend in Docker
docker-compose up -d backend

# Terminal 2: Frontend local for hot reload
cd frontend && bun run dev
```

**Proxy Configuration**:
- Local dev: Frontend dev server proxies `/api` and WebSocket to backend via Vite config
- Docker: Nginx proxies `/api/` and `/api/rooms/` to backend service
- Both setups handle client-side routing for Vue Router

---

## 🎨 Code Style & Conventions

### Frontend (Vue 3 + TypeScript)

**Core Stack**: Vue 3 (Composition API), TypeScript (non-strict), Tailwind CSS, Pinia, Vue Router

**Component Structure** (order matters):
```vue
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useGameStore } from '@/stores/game'
import type { GameMessage } from '@/types'

// Props with types
const props = defineProps<{
  title: string
  playerId?: string
}>()

// Emits
const emit = defineEmits<{
  join: [playerId: string]
}>()

// State, computed, methods...
const count = ref(0)
</script>

<template>
  <div class="p-4 bg-gray-900">
    <h1 class="text-xl font-bold">{{ title }}</h1>
  </div>
</template>
```

**TypeScript**: `strict: false` in `tsconfig.app.json`. Prefer explicit types but `any` is allowed.

**Styling**: Use Tailwind utility classes. Dynamic classes: `:class="['p-4', isActive && 'bg-blue-500']"`

### Backend (Rust)

**Core Stack**: Axum, Tokio, Serde, DashMap

**Module Structure**:
```rust
// lib.rs - module declarations
pub mod data;
pub mod filter;
pub mod server;

// Individual modules
use crate::data::*;
use crate::filter::CensorshipFilter;
```

**Key Patterns**:
- Types: Type aliases for IDs (`RoomId = String`)
- State: `AppState` with `Arc<RoomManager>`; `RoomConnector` shares `Arc<Mutex<ChatRoom>>`
- Censorship: Dynamic rules loaded from `words.json` via `words.rs`
- Async: `tokio::select!` for concurrent operations
- Serialization: `serde` with `rename_all = "snake_case"`

**Error Handling**: Use `Result<T, StatusCode>` in Axum handlers. Log with `eprintln!`.

---

## 📝 Naming Conventions

| Type | Pattern | Example |
|------|---------|---------|
| Vue Components | PascalCase.vue | `ChatBox.vue` |
| Vue Views | PascalCase.vue | `HomeView.vue` |
| Pinia Stores | camelCase.ts | `useGameStore.ts` |
| Rust Modules | snake_case.rs | `room.rs`, `filter.rs` |
| Rust Types | PascalCase | `RoomState`, `UserAction` |
| Rust Functions | snake_case | `process_action()` |
| Type Aliases | PascalCase | `RoomId`, `UserId` |
| Constants | UPPER_SNAKE | `MAX_USER_ACTIONS` |

---

## 🛠️ Error Handling & Debugging

### Frontend
```typescript
// Async operations always use try/catch
async function connect() {
  try {
    await socket.connect()
  } catch (error) {
    console.error('Connection failed:', error)
  }
}
```
- Remove `console.log` before commits
- Keep `console.error` for caught exceptions

### Backend
```rust
// Axum error handling
async fn handler() -> Result<Json<T>, StatusCode> {
    let data = get_data().ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(data))
}
```
- Use `eprintln!` for server errors
- Graceful degradation with defaults in `utils.rs`

---

## 📁 Project Structure

```
project-babel/
├── backend/               # Rust Axum server
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   ├── lib.rs         # Module declarations
│   │   ├── server.rs      # HTTP/WebSocket routes
│   │   ├── room.rs        # Chat room logic
│   │   ├── manager.rs     # Room lifecycle management
│   │   ├── filter.rs      # Censorship filter logic
│   │   ├── words.rs       # Word list loading & generation
│   │   ├── data.rs        # Types and structs
│   │   └── utils.rs       # File I/O helpers
│   ├── Cargo.toml
│   └── words.json         # Word lists (normal & censored)
├── frontend/              # Vue 3 frontend
│   ├── src/
│   │   ├── api/           # HTTP client
│   │   ├── components/    # Vue components
│   │   ├── composables/   # Vue composables
│   │   ├── router/        # Vue Router config
│   │   ├── stores/        # Pinia stores
│   │   ├── styles/        # Global styles
│   │   ├── types/         # TypeScript definitions
│   │   ├── views/         # Page components
│   │   ├── App.vue
│   │   ├── main.ts
│   │   └── style.css
│   ├── bun.lock           # Bun lockfile (text format)
│   ├── package.json
│   └── vite.config.ts
├── docs/                  # Documentation
├── Cargo.toml             # Workspace root
├── flake.nix              # Nix development environment
├── flake.lock
└── README.md
```

---

## ⚠️ Repository Quirks

1. **Docker First**: Docker Compose is the recommended way to run the project. All services are containerized for consistency.
2. **No Linting**: No ESLint/Prettier (frontend) or Clippy (backend). Follow conventions manually.
3. **Frontend Tests**: None configured. Manual verification required.
4. **Backend Tests**: Unit tests exist in `filter.rs`. Run with `cargo test`.
5. **TS Config**: `tsconfig.app.json` is active; `tsconfig.json` is a reference container.
6. **Package Manager**: `bun.lock` is the lockfile (text format). Both Bun and npm work, but Bun is ~4-6× faster.
7. **Node Modules**: Located in `frontend/node_modules/` after `bun install`.
8. **Port Mappings**:
   - Docker Production: Frontend `8080`, Backend `3000`
   - Local Development: Frontend `5173`, Backend `3000`

---

## 🚀 Agent Workflows

### Development Mode Selection

Before starting work, determine which development mode to use:

| Mode | Best For | Command |
|------|----------|---------|
| **Docker Compose** | Testing full integration, production parity | `docker-compose up -d` |
| **Local + Docker** | Frontend hot reload + stable backend | `docker-compose up -d backend` + `cd frontend && bun run dev` |
| **Full Local** | Maximum debugging capability, fast iteration | `cargo run` + `bun run dev` |

### General Guidelines

1. **Docker First**: When in doubt, use `docker-compose up -d for consistent environments
2. **Frontend Changes**: Prefer `bun` for frontend tasks (install, build, dev, lint, etc.). If `bun` is not found, fallback to `npm`. Work in container (`docker-compose up -d`) OR local (`cd frontend && bun run dev`)
3. **Backend Changes**: Can hot-reload with `docker-compose up -d` (rebuild on change) OR local `cargo run`
4. **Type Safety**: Run `bunx vue-tsc -b` after TS changes; `cargo check` after Rust changes
5. **Testing**: Describe manual verification steps since test coverage is minimal
6. **No Type Suppression**: Never use `@ts-ignore`, `as any`, or `unwrap_unchecked()`

---

## 📋 Quick Reference

| Task | Command |
|------|---------|
| Start all (Docker) | `docker-compose up -d` |
| Start both (Local) | Backend: `cargo run`, Frontend: `bun run dev` |
| Rebuild after changes | `docker-compose up -d --build` |
| View logs | `docker-compose logs -f` |
| Add Vue component | `src/components/ComponentName.vue` |
| Add Pinia store | `src/stores/feature.ts` with `useFeatureStore` |
| Add Rust module | Create `src/module.rs` + add to `lib.rs` |
| Backend test | `cargo test` or `cargo test filter::tests` |
