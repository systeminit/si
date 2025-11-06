# SI Function Tester

A testing framework for System Initiative functions that allows you to write
unit tests with mocked external dependencies.

## What is This?

SI Function Tester lets you test your System Initiative functions **without
needing AWS credentials or real cloud resources**. It mocks external
dependencies like `siExec` (AWS CLI calls) and `requestStorage`, so tests run
fast and reliably in isolation.

## Features

- ✅ **Mock AWS CLI calls** - No real AWS credentials needed
- ✅ **Fast execution** - Tests run in milliseconds
- ✅ **Type-safe** - Full TypeScript support
- ✅ **Convention-based** - Automatically pairs `*.test.ts` with `*.ts` files
- ✅ **All function types** - Actions, Management, Codegen, Qualifications,
  Attributes, Authentication
- ✅ **Persistent server** - Keep Docker container running for fast iterative
  testing
- ✅ **Simple command** - Single `si-test <directory>` command

## Quick Start

### 1. Download the `si-test` script

```bash
# Download the script
curl -O https://raw.githubusercontent.com/systeminit/si/main/bin/si-function-tester/si-test

# Make it executable
chmod +x si-test

# Optionally, move to your PATH
sudo mv si-test /usr/local/bin/
# OR for user-only install
mkdir -p ~/bin && mv si-test ~/bin/
```

### 2. Start the test server

```bash
# Pull and start the server (once per session)
docker run -d -p 8081:8081 --name si-function-tester systeminit/si-function-tester:stable

# Verify it's running
curl http://localhost:8081/health
```

### 3. Run tests

```bash
# Run tests on a directory (automatically finds test pairs)
si-test /path/to/your/functions

# Or test a specific pair
si-test examples/attribute
```

**Output:**

```
Scanning directory: examples/

Testing: attribute [attribute]
  ✓ returns most recent AMI matching filters (88.3ms)
  ✓ returns empty string when no filters specified (88.6ms)

Test Results: 2 passed, 0 failed
All tests passed!

SUMMARY
Total test pairs: 1
Passed: 1
✓ All test pairs passed!
```

## Convention-Based Testing

**File naming is important!** The test runner automatically pairs files based on a naming convention:

- `<name>.ts` - Your function implementation
- `<name>.test.ts` - Your test suite

Both files must be in the same directory. For example:
```
my-functions/
  ├── create.ts          ← Function
  ├── create.test.ts     ← Tests for create.ts
  ├── update.ts          ← Function
  └── update.test.ts     ← Tests for update.ts
```

When you run `./si-test my-functions/`, it will:
1. Find all `*.test.ts` files recursively
2. Look for the corresponding `*.ts` file in the same directory
3. Run tests for each pair found

**Skipped pairs:** If a `.test.ts` file exists without a matching `.ts` file, it will be skipped with a warning.

## API Reference

### `defineTests(tests: TestSuite)`

Define a test suite with one or more test cases.

```typescript
export default defineTests({
  "test name": {
    input: { properties: { /* ... */ } },
    mocks: { /* ... */ },
    expect: { /* ... */ },
    timeout?: number,
    skip?: boolean
  }
});
```

### Test Case Structure

```typescript
interface TestCase {
  // Input to your function
  input: {
    properties?: Record<string, unknown>;
    [key: string]: unknown;
  };

  // Mock configurations
  mocks?: {
    exec?: ExecMockBuilder;
    storage?: Record<string, unknown>;
  };

  // Expected output
  expect: {
    status?: "ok" | "warning" | "error" | Matcher<string>;
    payload?: unknown;
    resourceId?: string | null | Matcher<string | null>;
    message?: string | Matcher<string>;
    validate?: (result: ActionResult) => boolean | void;
  };

  // Test timeout in milliseconds (default: 5000)
  timeout?: number;

  // Skip this test
  skip?: boolean;
}
```

### Mocking `siExec`

Mock external command execution:

```typescript
mocks: {
  exec: mockExec()
    .command("aws s3 mb") // String match
    .returns({
      stdout: "make_bucket: my-bucket",
      stderr: "",
      exitCode: 0,
    })
    .command(/terraform apply/) // Regex match
    .returns({ stdout: "Applied", exitCode: 0 })
    .command("failing command")
    .throws(new Error("Command failed"));
}
```

**Sequential mocks** (for multiple calls to same command):

```typescript
mocks: {
  exec: mockExec()
    .command("aws cloudcontrol get-resource-request-status")
    .returns({ stdout: '{"Status":"IN_PROGRESS"}', exitCode: 0 })
    .command("aws cloudcontrol get-resource-request-status")
    .returns({ stdout: '{"Status":"SUCCESS"}', exitCode: 0 });
}
```

### Matchers

Use matcher functions for flexible assertions:

```typescript
expect: {
  status: "ok",
  message: expect.stringContaining("success"),
  resourceId: expect.string(),
  payload: expect.objectContaining({ id: "123" })
}
```

**Available matchers:**

- `expect.string()` - Any string
- `expect.stringContaining(substring)` - String containing substring
- `expect.stringMatching(regex)` - String matching regex
- `expect.object()` - Any object
- `expect.objectContaining(keys)` - Object containing specific keys
- `expect.number()` - Any number
- `expect.boolean()` - Any boolean
- `expect.array()` - Any array
- `expect.null()` - null
- `expect.undefined()` - undefined
- `expect.any()` - Any value

### Custom Validation

Write custom validation logic for complex checks:

```typescript
expect: {
  validate: (result) => {
    const payload = result.payload as { arn: string };

    if (!payload.arn.startsWith("arn:aws:s3:::")) {
      throw new Error("Invalid S3 ARN format");
    }

    // Returning nothing or true means validation passed
  };
}
```

### Mocking `requestStorage`

Pre-populate storage for stateful tests:

```typescript
mocks: {
  storage: {
    previousValue: 42,
    operationId: "op-123"
  }
}
```

Access storage in validation:

```typescript
expect: {
  validate: (result) => {
    const storage = requestStorage._inspect();
    if (storage.data.counter !== 43) {
      throw new Error("Counter not incremented");
    }
  };
}
```

## Examples

See complete working examples in [`examples/`](examples/):

- **actions/** - Action functions (create, update, delete resources) with
  retries and polling
- **attribute/** - Attribute functions (compute derived values from component
  properties)
- **authentication/** - Authentication functions (set credentials and
  environment variables)
- **codegen/** - Codegen functions (generate CloudControl API payloads)
- **management/** - Management functions (import, refresh resources)
- **qualifications/** - Qualification functions (validate component state)

**Run all examples:**

```bash
./si-test examples/
```

This will run 26 tests across all 6 function types!

## Import Path

Always use this import path in your test files:

```typescript
import { defineTests, mockExec, expect } from "file:///app/index.ts";
```

This works both in Docker and when running tests with Deno directly.

## Documentation

- **[USAGE.md](USAGE.md)** - Build and usage instructions

## License

See LICENSE in the root of the SI repository.
