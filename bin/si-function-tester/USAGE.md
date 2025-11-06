# SI Function Tester - Usage Guide

## Quick Start

### 1. Download the `si-test` Script

```bash
# Download the script from GitHub
curl -O https://raw.githubusercontent.com/systeminit/si/main/bin/si-function-tester/si-test

# Make it executable
chmod +x si-test
```

### 2. Start the Test Server

**Recommended: Use pre-built image from Docker Hub**

```bash
# Pull and start the server
docker run -d -p 8081:8081 --name si-function-tester systeminit/si-function-tester:stable

# Verify it's running
curl http://localhost:8081/health
```

**Alternative: Build locally (for development)**

```bash
# From the bin/si-function-tester directory
cd bin/si-function-tester
docker build -t si-function-tester .

# Start the server
docker run -d -p 8081:8081 --name si-function-tester si-function-tester

# Verify it's running
curl http://localhost:8081/health
```

### 3. Install the `si-test` Command (Optional)

```bash
# Install globally (requires sudo)
sudo mv si-test /usr/local/bin/

# OR install to user bin (no sudo)
mkdir -p ~/bin
mv si-test ~/bin/
# Ensure ~/bin is in your PATH
```

### 4. Run Tests

```bash
si-test ./my-tests-directory
```

That's it! The command automatically discovers all `*.test.ts` files and pairs
them with their corresponding function files.

---

## Build Instructions

### Option 1: Use Pre-built Image (Recommended)

```bash
# Pull from Docker Hub
docker pull systeminit/si-function-tester:stable
```

### Option 2: Build from GitHub

```bash
# Build directly from GitHub
docker build -t si-function-tester https://github.com/systeminit/si.git#main:bin/si-function-tester
```

### Option 3: Build Locally (Development)

```bash
# From the bin/si-function-tester directory
cd bin/si-function-tester
docker build -t si-function-tester .
```

The Docker image includes:

- Deno runtime with TypeScript support
- All test framework code
- Health check endpoint
- Runs on port 8081

### Option 4: Standalone Deno (No Docker)

No build required - run directly from the source:

```bash
# From bin/si-function-tester directory
deno run --allow-all server.ts
```

---

## Usage Instructions

### Running the Server

**Start (using pre-built image):**

```bash
docker run -d -p 8081:8081 --name si-function-tester systeminit/si-function-tester:stable
```

**Start (using locally built image):**

```bash
docker run -d -p 8081:8081 --name si-function-tester si-function-tester
```

**Check health:**

```bash
curl http://localhost:8081/health
# {"status":"ok","timestamp":1699999999999}
```

**View logs:**

```bash
docker logs si-function-tester
docker logs -f si-function-tester  # Follow logs
```

**Restart:**

```bash
docker restart si-function-tester
```

**Stop:**

```bash
docker stop si-function-tester
```

**Remove:**

```bash
docker stop si-function-tester
docker rm si-function-tester
```

### Running Tests

#### Using `si-test` (Recommended)

```bash
# Test all pairs in a directory (automatically searches subdirectories)
si-test ./tests

# Test entire project
si-test ./my-project

# Use remote server
si-test -s http://remote:8081 ./tests

# JSON output (for CI/CD)
si-test --json ./tests > results.json

# Help
si-test --help
```

**Environment variable:**

```bash
export SI_TEST_SERVER=http://remote:8081
si-test ./tests
```

**Convention:** The command automatically discovers test pairs recursively:

- Searches the directory and all subdirectories for `*.test.ts` files
- Pairs them with corresponding function files (e.g., `create.test.ts` →
  `create.ts`)
- Runs all pairs in sequence

---

## Writing Tests

### Test File Structure

```typescript
import { defineTests, mockExec, expect } from "file:///app/index.ts";

export default defineTests({
  "test name": {
    input: {
      properties: {
        /* your input */
      },
    },
    mocks: {
      exec: mockExec()
        .command("aws s3 mb")
        .returns({ stdout: "success", exitCode: 0 }),
    },
    expect: {
      status: "ok",
      resourceId: "my-bucket",
    },
  },
});
```

### Import Path

**Always use this import path:**

```typescript
import { defineTests, mockExec, expect } from "file:///app/index.ts";
```

This works in Docker and with Deno directly.

---

## Workflow Examples

### Daily Development Workflow

```bash
# Start server once (beginning of day)
docker run -d -p 8081:8081 --name si-function-tester systeminit/si-function-tester:stable

# Write your functions and tests in a directory
# ... edit files ...

# Run tests as you develop (many times, fast!)
si-test ./my-project/actions  # Just the actions directory
si-test ./my-project          # Entire project (all subdirectories)

# Stop server (end of day)
docker stop si-function-tester
```

### CI/CD Workflow

```yaml
# .github/workflows/test.yml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Start test server
        run: |
          docker run -d -p 8081:8081 --name si-function-tester systeminit/si-function-tester:stable

      - name: Wait for server
        run: |
          timeout 30 bash -c 'until curl -f http://localhost:8081/health; do sleep 1; done'

      - name: Install si-test
        run: |
          curl -O https://raw.githubusercontent.com/systeminit/si/main/bin/si-function-tester/si-test
          chmod +x si-test
          sudo mv si-test /usr/local/bin/

      - name: Run tests
        run: |
          si-test ./my-functions

      - name: Cleanup
        if: always()
        run: docker stop si-function-tester
```

### Watch Mode

Create `watch-tests.sh`:

```bash
#!/bin/bash

# Ensure server is running
if ! docker ps | grep -q si-function-tester; then
  docker run -d -p 8081:8081 --name si-function-tester systeminit/si-function-tester:stable
fi

# Watch for changes and run tests
while true; do
  inotifywait -r -e modify ./tests
  clear
  si-test ./tests
done
```

---

## Advanced Configuration

### Custom Port

```bash
# Run server on different port
docker run -d -p 9090:8081 --name si-function-tester systeminit/si-function-tester:stable

# Use custom port with si-test
si-test -s http://localhost:9090 ./tests
```

### Resource Limits

```bash
docker run -d -p 8081:8081 \
  --name si-function-tester \
  --cpus="2" \
  --memory="2g" \
  systeminit/si-function-tester:stable
```

### Using with Remote Servers

```bash
# Start server on remote machine
ssh remote-host 'docker run -d -p 8081:8081 --name si-function-tester systeminit/si-function-tester:stable'

# Run tests from local machine
si-test -s http://remote-host:8081 ./tests
```

---

## Troubleshooting

### Server won't start

**Check if port is in use:**

```bash
lsof -i :8081
```

**Use different port:**

```bash
docker run -d -p 9090:8081 --name si-function-tester systeminit/si-function-tester:stable
si-test -s http://localhost:9090 ./tests
```

### Connection refused

**Check server is running:**

```bash
docker ps | grep si-function-tester
```

**Check logs:**

```bash
docker logs si-function-tester
```

**Restart server:**

```bash
docker restart si-function-tester
```

### Tests timing out

**Increase timeout in test:**

```typescript
{
  input: { /* ... */ },
  expect: { /* ... */ },
  timeout: 10000  // 10 seconds
}
```

**Check server resources:**

```bash
docker stats si-function-tester
```

### Import errors

Make sure you're using the correct import path:

```typescript
// ✅ Correct
import { defineTests } from "file:///app/index.ts";

// ❌ Wrong
import { defineTests } from "../../src/test-framework/index.ts";
```

---

## Performance

| Method                     | Speed                   |
| -------------------------- | ----------------------- |
| Persistent Server (Docker) | ~0.1-0.5s per test ⚡   |
| Deno CLI (local)           | ~0.1-0.3s per test ⚡⚡ |

**Tip:** Keep the server running during development for fastest iteration.

---

## Requirements

### To Run the Server

- Docker

### To Run Tests

- **Option 1**: `si-test` script (requires: `curl`, `jq`)
- **Option 2**: `client.ts` (requires: Deno)
- **Option 3**: Direct Deno (requires: Deno)

---

## Examples

See working examples in [`examples/tutorial/`](examples/tutorial/):

- `hello-world.ts` / `hello-world.test.ts` - Basic example
- `s3-bucket.ts` / `s3-bucket.test.ts` - AWS CLI mocking
- `cloudcontrol-create.test.ts` - Complex real-world example

Run them:

```bash
si-test examples/tutorial/hello-world.ts examples/tutorial/hello-world.test.ts
si-test examples/tutorial/s3-bucket.ts examples/tutorial/s3-bucket.test.ts
```

---

## Summary

**Start server once per session:**

```bash
docker run -d -p 8081:8081 --name si-function-tester systeminit/si-function-tester:stable
```

**Run tests many times (fast!):**

```bash
si-test ./tests
```

**Stop when done:**

```bash
docker stop si-function-tester
```

That's it! 🚀

**No local build needed** - the image is pre-built and available on Docker Hub!
