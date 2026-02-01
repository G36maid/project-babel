# 🗼 Project Babel - Frontend

**Framework:** Vue 3 (Composition API) + TypeScript
**Styling:** Tailwind CSS
**State:** Pinia

## 🚀 Quick Start (Recommended)

The most recommended way to run the frontend is via Docker Compose from the root directory:

```bash
docker-compose up -d frontend
```

For local development with Hot Module Replacement (HMR):

```bash
bun install
bun run dev
```

## 📂 Structure

- `src/components/`: Reusable Vue components.
- `src/views/`: Page-level components (`HomeView`, `GameView`).
- `src/stores/`: Pinia stores for state management.
- `src/api/`: API client and WebSocket connection logic.
- `src/types/`: TypeScript interfaces and types.
- `src/assets/`: Static assets including the custom symbol SVGs.

## 🎨 Symbols

The game uses a custom set of 26 symbols for communication. These are located in `src/assets/symbols/`.