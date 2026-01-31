# Multi-stage build for Vue 3 frontend
# Stage 1: Build stage
FROM docker.io/oven/bun:1-alpine AS builder

WORKDIR /app

# Copy package files
COPY package.json bun.lock ./

# Install dependencies
RUN bun install --frozen-lockfile

# Copy source code
COPY . .

# Build the application
RUN bun run build

# Stage 2: Production stage with Nginx
FROM docker.io/nginx:alpine AS production

COPY --from=builder /app/dist /usr/share/nginx/html

COPY ./nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]

# Stage 3: Development stage (optional, for hot reload)
FROM docker.io/oven/bun:1-alpine AS development

WORKDIR /app

# Copy package files
COPY package.json bun.lock ./

# Install all dependencies (including dev)
RUN bun install

# Copy source code
COPY . .

# Expose Vite dev server port
EXPOSE 5173

# Start dev server with host binding for Docker
CMD ["bun", "run", "dev", "--", "--host", "0.0.0.0"]
