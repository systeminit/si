# System Initiative TypeScript SDK Tests

Test coverage for the System Initiative TypeScript SDK (`system-initiative-api-client`).

## Setup

1. Install dependencies:
```bash
npm install
```

2. Configure environment variables:
```bash
cp .env.example .env
```

3. Edit `.env` with your workspace ID and API token:
   - Get your workspace ID and API token from https://auth.systeminit.com/workspaces/
   - Set `SI_WORKSPACE_ID` to your workspace ID
   - Set `SI_API_TOKEN` to your API token

## Running Tests

Run all tests:
```bash
npm test
```

Run tests once (CI mode):
```bash
npm run test:run
```

Run tests with coverage:
```bash
npm run test:coverage
```

## Environment Variables

- `SI_WORKSPACE_ID` - Your System Initiative workspace ID (required)
- `SI_API_TOKEN` - Your System Initiative API token (required)
- `SI_API_BASE_PATH` - API base URL (default: https://api.systeminit.com)

## Test Coverage

- **Change Sets API** (`change-sets.test.ts`)
  - Create a new change set
  - List all change sets
