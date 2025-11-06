# VPC Function Examples

This directory contains complete working examples demonstrating all SI function types.

## Running the Examples

Test all examples at once:
```bash
../si-test .
```

Test a specific function type:
```bash
../si-test actions      # Action functions
../si-test attribute    # Attribute functions
../si-test authentication  # Authentication functions
../si-test codegen      # Codegen functions
../si-test management   # Management functions
../si-test qualifications  # Qualification functions
```

## Function Types

### 1. Actions (`actions/`)
**Purpose:** Create, update, or delete cloud resources

**Example:** `create.ts` - Creates AWS resources via CloudControl API
- Polls for completion with exponential backoff
- Handles rate limiting with retries
- Returns resource ID on success

**Tests:** 10 tests covering success cases, error handling, retries, and edge cases

### 2. Attributes (`attribute/`)
**Purpose:** Compute derived values from component properties

**Example:** `attribute.ts` - Finds most recent AMI matching filters
- Queries AWS EC2 to find images
- Returns the most recent matching image ID
- Returns empty string when no query specified

**Tests:** 2 tests covering query results and empty inputs

### 3. Authentication (`authentication/`)
**Purpose:** Set credentials and environment variables for AWS access

**Example:** `auth.ts` - Configures AWS credentials
- Sets access keys directly or via assume role
- Configures session tokens and endpoint URLs
- Handles STS assume role with external ID

**Tests:** 5 tests covering direct credentials, assume role, and error cases

### 4. Codegen (`codegen/`)
**Purpose:** Generate CloudControl API payloads from component properties

**Example:** `code.ts` - Builds CloudControl create payload
- Filters properties based on PropUsageMap
- Includes secrets from requestStorage
- Removes empty/unused properties
- Returns JSON payload

**Tests:** 3 tests covering payload generation, secrets, and error handling

### 5. Management (`management/`)
**Purpose:** Import or refresh resources from cloud providers

**Example:** `import.ts` - Imports existing AWS resources
- Fetches resource details via CloudControl
- Updates component properties with live data
- Manages action availability (add/remove refresh/create)

**Tests:** 3 tests covering successful import, missing resourceId, and AWS errors

### 6. Qualifications (`qualifications/`)
**Purpose:** Validate that component configuration is correct

**Example:** `qualifications.ts` - Validates AMI configuration
- Checks that ImageId matches query results
- Validates region is specified
- Handles multiple matches with UseMostRecent flag

**Tests:** 3 tests covering success, validation failures, and missing data

## Test Structure

**File naming convention is critical!** Each function type follows this pattern:
- `<name>.ts` - The function implementation
- `<name>.test.ts` - The test suite

**Both files must be in the same directory** and the names must match exactly (except for the `.test` suffix). For example:
```
actions/
  ├── create.ts       ← Function implementation
  └── create.test.ts  ← Tests for create.ts (name matches!)
```

The test runner automatically discovers and pairs these files when scanning directories.

Tests use the pattern:
```typescript
import { defineTests, mockExec } from "file:///app/index.ts";

export default defineTests({
  "test name": {
    input: { /* function input */ },
    mocks: { /* mocked dependencies */ },
    expect: { /* expected output or validation */ }
  }
});
```

## Key Patterns Demonstrated

1. **Mocking AWS CLI calls** - See all examples
2. **Sequential mocks** - `actions/create.test.ts` (polling)
3. **Custom validation** - `qualifications/qualifications.test.ts`
4. **RequestStorage** - `codegen/code.test.ts` (secrets)
5. **Error handling** - All examples
6. **Retries and rate limiting** - `actions/create.test.ts`
