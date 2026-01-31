# 🤖 AGENTS.md - Project Babel Guidelines

> **Context**: This is a Vue 3 + TypeScript + Tailwind CSS frontend project.
> Note: The project structure implies a Rust backend, but currently only the `frontend/` directory is present/active in this context.

## 🏗️ Build & Run Commands

**Working Directory**: All commands must be run from the `frontend/` directory.

| Action | Command | Description |
| :--- | :--- | :--- |
| **Install** | `npm install` | Install dependencies (uses `package-lock.json` if present, else checks `bun.lock`) |
| **Dev Server** | `npm run dev` | Starts Vite dev server (default: http://localhost:5173) |
| **Build** | `npm run build` | Runs type check (`vue-tsc`) and builds for production |
| **Type Check** | `npx vue-tsc -b` | Runs TypeScript compiler in build mode |
| **Test** | *None* | No testing framework is currently configured. |

> **Important**: There is NO `package.json` in the project root. Always `cd frontend` before running npm commands.

---

## 🎨 Code Style & Conventions

### Core Stack
- **Framework**: Vue 3 (Composition API with `<script setup lang="ts">`)
- **Language**: TypeScript (Configured as **non-strict**)
- **Styling**: Tailwind CSS
- **State**: Pinia
- **Routing**: Vue Router

### TypeScript Rules
- **Strictness**: `strict: false` is set in `tsconfig.app.json`.
  - While type safety is encouraged, do not break existing code by enforcing strict null checks or strict property initialization unless you are refactoring.
  - Interfaces/Types should be defined in the same file if local, or in `types/` if shared.
- **Explicit Types**: meaningful types are preferred over `any`, but the config allows implicit `any`.

### Component Structure (`.vue`)
Use the `<script setup>` syntax. Order blocks as:
1. `<script setup lang="ts">`
2. `<template>`
3. `<style>` (scoped not mandatory if using Tailwind, but often used for specific overrides)

```vue
<script setup lang="ts">
import { ref, computed } from 'vue'
import type { User } from '@/types'

// Props
const props = defineProps<{
  title: string
}>()

// State
const count = ref(0)
</script>

<template>
  <div class="p-4 bg-gray-900">
    <h1 class="text-xl font-bold">{{ title }}</h1>
  </div>
</template>
```

### Styling (Tailwind)
- Use utility classes primarily.
- Avoid `@apply` in CSS files unless creating a reusable component class that is used in multiple places.
- Dynamic classes: Use array syntax or template literals: `:class="['p-4', isActive ? 'bg-blue-500' : 'bg-gray-500']"`

### Naming Conventions
- **Files**:
  - Components: `PascalCase.vue` (e.g., `ChatBox.vue`)
  - Utilities: `camelCase.ts` or `kebab-case.ts`
- **Variables/Functions**: `camelCase`
- **Stores**: `useStoreName` (e.g., `useAuthStore`)

---

## 🛠️ Error Handling & Debugging

- **Async/Await**: Always use `try/catch` blocks for async operations (API calls).
- **Console**: Clean up `console.log` before committing, but `console.error` is acceptable for caught errors.
- **Vue**: Use `onErrorCaptured` for component-level error handling if needed.

## ⚠️ Repository Quirks
- **Missing Backend**: The README mentions a Rust backend, but it may not be present in the working tree. Focus on Frontend tasks.
- **Config Files**: `tsconfig.app.json` controls the app configuration. `tsconfig.json` is just a reference container.
- **Deps**: `node_modules` exists in root but `package.json` does not. Trust `frontend/package.json`.

---

## 🚀 Workflows for Agents
1. **Always Check Path**: Verify you are in `frontend/` before running scripts.
2. **Read Configs**: Check `vite.config.ts` and `tailwind.config.js` before making infrastructure changes.
3. **No Tests**: Since there are no tests, **manual verification** (via reasoning or asking user) is critical for logic changes.
