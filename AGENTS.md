# 🤖 AGENTS.md - Project Babel Guidelines

> **Context**: Multiplayer puzzle game. Vue 3 (Frontend) + Rust Axum (Backend).
> **Goal**: Players use a symbolic language to communicate across censorship firewalls.

## 🏗️ Build & Run Commands

**Working Directory**: Project root (`/home/g36maid/Code/GameDev/project-babel`)

| Action | Command | Description |
|--------|---------|-------------|
| **Start All** | `docker-compose up -d` | **Recommended**. Starts everything. |
| **Logs** | `docker-compose logs -f` | View logs. |
| **Rebuild** | `docker-compose up -d --build` | Rebuild containers. |
| **Frontend Dev** | `cd frontend && bun run dev` | Local Dev (`localhost:5173`). Fast. |
| **Backend Dev** | `cd backend && cargo run` | Local Dev (`localhost:3000`). |
| **Backend Test** | `cargo test` | Run all tests. |
| **Single Test** | `cargo test <test_name>` | Run specific Rust test. |
| **Type Check** | `bunx vue-tsc -b` | Frontend type check. |
| **Lint (Rust)** | `cargo clippy` | Run Rust linter. |

*Note: Frontend has no configured test runner (Vitest/Jest). Manual verification required.*

## 🎨 Code Style & Conventions

### Frontend (Vue 3 + TypeScript)
- **Stack**: Vue 3 (Composition API), TypeScript (non-strict), Tailwind CSS, Pinia.
- **Strictness**: `strict: false` in `tsconfig`. Prefer types, but `any` is allowed if blocked.
- **Styling**: Tailwind utility classes. `class="p-4 bg-gray-900"`.
- **Store**: Pinia stores in `src/stores/`. Use `setup` syntax for stores.
- **Linting & Formatting**: **Biome** (Unified Tool).
  - Check & Auto-fix: `bun run check`
  - Lint only: `bun run lint`
  - Format only: `bun run format`
- **Components**: `<script setup lang="ts">`. Order: imports, props/emits, logic.

```vue
<script setup lang="ts">
import { ref } from 'vue';
defineProps<{ title: string }>();
</script>
<template>
  <div class="p-4">{{ title }}</div>
</template>
```

### Backend (Rust + Axum)
- **Stack**: Axum, Tokio, Serde, DashMap.
- **Patterns**:
  - **Errors**: `Result<Json<T>, StatusCode>`. Log with `eprintln!`.
  - **State**: Shared state via `Arc<AppState>`.
  - **Async**: `tokio::select!` for concurrency.
  - **Serialization**: `serde` with `rename_all = "snake_case"`.

```rust
async fn handler() -> Result<Json<Data>, StatusCode> {
    let data = compute().ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(data))
}
```

## 📝 Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| **Vue Files** | PascalCase | `ChatBox.vue` |
| **TS Files** | camelCase | `useGameStore.ts` |
| **Rust Modules** | snake_case | `room.rs` |
| **Rust Types** | PascalCase | `RoomState` |
| **Rust Consts** | UPPER_SNAKE | `MAX_USERS` |

## 📁 Key Structure

- `backend/src/`
  - `server.rs`: HTTP/WS routes.
  - `room.rs`: Core game logic.
  - `filter.rs`: Censorship logic (contains unit tests).
- `frontend/src/`
  - `components/`: Vue components.
  - `stores/`: Pinia state.
  - `views/`: Page routes.

## ⚠️ Agent Behaviors
1.  **Docker First**: Default to `docker-compose` for running the app.
2.  **Bun**: Use `bun` for frontend (faster than npm).
3.  **No Type Suppression**: Avoid `@ts-ignore` unless absolutely necessary.
4.  **Verification**:
    - **Rust**: `cargo check` && `cargo clippy` && `cargo fmt`.
    - **Vue**: `bunx vue-tsc -b` && `bun run check` (Biome).
    - **General**: `yamllint .` (Check YAML syntax).
    - **Logic**: Verify manually via browser or `curl` if no auto-tests exist.

## 🌿 Branch & Commit Strategy

### Conventional Commits
Format: `type(scope): description`

- **Types**:
  - `feat`: New feature (e.g., `feat(room): add voice input`)
  - `fix`: Bug fix (e.g., `fix(chat): repair message scroll`)
  - `docs`: Documentation only
  - `style`: Formatting, missing semi-colons, etc.
  - `refactor`: Code change that neither fixes a bug nor adds a feature
  - `perf`: Code change that improves performance
  - `test`: Adding missing tests or correcting existing tests
  - `chore`: Build process, auxiliary tools, deps updates
- **Scope** (Optional): `frontend`, `backend`, `docker`, `ci`, `room`, `auth`

### Linear Branch Management
Format: `type/kebab-case-description`

- **Pattern**: `feat/voice-input`, `fix/login-bug`, `chore/upgrade-deps`
- **Workflow**:
  1.  Create branch from `main`.
  2.  Make atomic commits.
  3.  Open PR (Squash & Merge recommended).
  4.  Delete branch after merge.

### Git Rules
- **No Force Push** to shared branches (`main`).
- **Atomic Commits**: Each commit should do one thing well.
- **Sync**: Rebase on `main` frequently to avoid conflicts.
