# System Initiative Python SDK Tests

Test coverage for the System Initiative Python SDK (`system-initiative-api-client`). These tests validate API client functionality against a live System Initiative workspace.

## Setup

### Prerequisites

- Python 3.8 or higher
- pip

### Quick Setup

Run the setup script (Linux/Mac):
```bash
./setup.sh
```

Then edit `.env` with your credentials from https://auth.systeminit.com/workspaces/

### Manual Setup

1. Create and activate a virtual environment:
```bash
python3 -m venv .venv
source .venv/bin/activate  # On Linux/Mac
# OR
.venv\Scripts\activate  # On Windows
```

2. Install dependencies:
```bash
pip install -e ".[dev]"
```

3. Configure environment variables:
```bash
cp .env.example .env
# Edit .env with your workspace ID and API token from https://auth.systeminit.com/workspaces/
```

## Running Tests

**Note:** Make sure your virtual environment is activated first:
```bash
source .venv/bin/activate
```

Run all tests:
```bash
pytest
```

Run tests with verbose output:
```bash
pytest -v
```

Run a specific test file:
```bash
pytest tests/test_change_sets.py
```

Run tests with coverage:
```bash
pytest --cov=tests --cov-report=html
```

## Environment Variables

- `SI_WORKSPACE_ID` - Your System Initiative workspace ID (required)
- `SI_API_TOKEN` - Your API authentication token (required)
- `SI_API_BASE_PATH` - API endpoint URL (default: https://api.systeminit.com)

Get your workspace ID and API token from [https://auth.systeminit.com/workspaces/](https://auth.systeminit.com/workspaces/)

## Test Structure

- `tests/test_change_sets.py` - Tests for Change Sets API
- `tests/test_secrets.py` - Tests for Secrets API

Each test suite creates its own change sets to isolate test resources and validates both API responses and data structures.

## SDK Documentation

For more information about the System Initiative Python SDK, see:
- [Public API Documentation](https://docs.systeminit.com/reference/public-api)
- [Python SDK on PyPI](https://pypi.org/project/system-initiative-api-client/)
