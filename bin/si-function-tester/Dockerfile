# Runtime stage - use deno alpine image for compatibility
FROM denoland/deno:alpine

LABEL org.opencontainers.image.title="SI Function Tester Server"
LABEL org.opencontainers.image.description="HTTP service for testing SI functions"
LABEL org.opencontainers.image.vendor="System Initiative"
LABEL org.opencontainers.image.source="https://github.com/systeminit/si"

WORKDIR /app

# Copy framework files needed at runtime
COPY types.ts /app/types.ts
COPY mocks.ts /app/mocks.ts
COPY runner.ts /app/runner.ts
COPY index.ts /app/index.ts
COPY server.ts /app/server.ts
COPY deno.json /app/deno.json

# Expose HTTP port
EXPOSE 8081

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:8081/health || exit 1

# Set the entrypoint to run the server with deno
ENTRYPOINT ["deno", "run", "--allow-all", "server.ts"]
