# 🤖 AGENTS.md - Project Babel Guidelines

> **Context**: A multiplayer puzzle game with Vue 3 frontend and Rust (Axum) backend.  
> **Game**: Players use a symbolic language to communicate across censorship firewalls.

---

## 🏗️ Build & Run Commands

### Frontend (Vue 3 + TypeScript)
**Working Directory**: `frontend/`

| Action | Bun (Primary) | npm (Alternative) | Description |
|--------|---------------|-------------------|-------------|
| Install | `bun install` | `npm install` | Install dependencies |
| Dev | `bun run dev` | `npm run dev` | Vite dev server on `localhost:5173` |
| Build | `bun run build` | `npm run build` | Type check + production build |
| Preview | `bun run preview` | `npm run preview` | Preview production build |
| Type Check | `bunx vue-tsc -b` | `npx vue-tsc -b` | TypeScript compiler only |

**Lockfile**: `bun.lock` (text format, git-diffable)  
**Why Bun?** ~4-6× faster than npm. Both work since they share `package.json`.

### Backend (Rust + Axum)
**Working Directory**: `backend/` or project root

| Action | Command | Description |
|--------|---------|-------------|
| Run | `cargo run` | Start server on `localhost:3000` |
| Build | `cargo build --release` | Production build |
| Test | `cargo test` | Run all tests |
| Test Single | `cargo test test_name` | Run specific test |
| Check | `cargo check` | Fast compile check |

### Development Workflow
```bash
# Terminal 1: Start backend
cd backend && cargo run

# Terminal 2: Start frontend (use Bun for speed)
cd frontend && bun run dev
```

**Proxy**: Frontend dev server proxies `/api` and `/socket.io` to backend.

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
- State: `AppState` with `Arc<RoomManager>`
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
│   │   ├── filter.rs      # Censorship filter + tests
│   │   ├── data.rs        # Types and structs
│   │   └── utils.rs       # File I/O helpers
│   ├── Cargo.toml
│   ├── filter_config.json # Censorship rules
│   └── user_tokens.json   # Auth tokens
├── frontend/              # Vue 3 frontend
│   ├── src/
│   │   ├── views/         # Page components
│   │   ├── stores/        # Pinia stores
│   │   ├── router/        # Vue Router config
│   │   ├── App.vue
│   │   └── main.ts
│   ├── bun.lock           # Bun lockfile (text format)
│   ├── package.json
│   └── vite.config.ts
├── Cargo.toml             # Workspace root
└── README.md
```

---

## ⚠️ Repository Quirks

1. **No Linting**: No ESLint/Prettier (frontend) or Clippy (backend). Follow conventions manually.
2. **Frontend Tests**: None configured. Manual verification required.
3. **Backend Tests**: Unit tests exist in `filter.rs`. Run with `cargo test`.
4. **TS Config**: `tsconfig.app.json` is active; `tsconfig.json` is a reference container.
5. **Package Manager**: `bun.lock` is the lockfile (text format). Both Bun and npm work, but Bun is ~4-6× faster.
6. **Node Modules**: Located in `frontend/node_modules/` after `bun install`.

---

## 🚀 Agent Workflows

1. **Frontend Changes**: Always `cd frontend/` first
2. **Backend Changes**: Can run from root (workspace) or `backend/`
3. **Type Safety**: Run `bunx vue-tsc -b` after TS changes; `cargo check` after Rust changes
4. **Testing**: Describe manual verification steps since test coverage is minimal
5. **No Type Suppression**: Never use `@ts-ignore`, `as any`, or `unwrap_unchecked()`

---

## 📋 Quick Reference

| Task | Command |
|------|---------|
| Start both services | Backend: `cargo run`, Frontend: `bun run dev` |
| Add Vue component | `src/components/ComponentName.vue` |
| Add Pinia store | `src/stores/feature.ts` with `useFeatureStore` |
| Add Rust module | Create `src/module.rs` + add to `lib.rs` |
| Backend test | `cargo test` or `cargo test filter::tests` |
