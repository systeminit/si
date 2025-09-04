# Build stage
FROM denoland/deno:2.4.2 AS builder

WORKDIR /app

# Copy source files
COPY . .

WORKDIR /app/bin/si-mcp-server

# Install dependencies
RUN deno install

# Compile the application
RUN deno compile --allow-env --allow-net --output=si-mcp-server main.ts

# Runtime stage - use deno alpine image for compatibility
FROM denoland/deno:alpine

# Copy the compiled binary from builder stage
COPY --from=builder /app/bin/si-mcp-server/si-mcp-server /si-mcp-server

# Set the entrypoint to run the binary with stdio argument
ENTRYPOINT ["/si-mcp-server", "stdio"]
