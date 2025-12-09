# GCP Scripts

Utility scripts for managing Google Cloud Platform resources.

## enable-gcp-apis.sh

Script to enable all available GCP APIs for a project.

### Prerequisites

- [gcloud CLI](https://cloud.google.com/sdk/docs/install) installed
- Appropriate permissions on the target GCP project

### Setup

1. **Authenticate with GCP**
   ```bash
   gcloud auth login
   ```

2. **List your available projects**
   ```bash
   gcloud projects list
   ```
   This shows all projects you have access to with their PROJECT_ID, NAME, and PROJECT_NUMBER.

3. **Set your active project**
   ```bash
   gcloud config set project PROJECT_ID
   ```
   Replace `PROJECT_ID` with the ID from the list above.

4. **Run the script**
   ```bash
   # Uses the project you just set
   ./enable-gcp-apis.sh

   # Or specify a different project
   ./enable-gcp-apis.sh --project PROJECT_ID
   ```

### Usage

```bash
# Basic usage - uses configured project
./enable-gcp-apis.sh

# Dry run to see what would be enabled
./enable-gcp-apis.sh --dry-run

# Or specify project explicitly
./enable-gcp-apis.sh --project your-project-id

# Enable with custom batch size (faster but may hit rate limits)
./enable-gcp-apis.sh --batch-size 20

# Show help
./enable-gcp-apis.sh --help
```

### Options

- `-p, --project`: GCP Project ID (optional - uses gcloud configured project if not provided)
- `-d, --dry-run`: Show what would be enabled without actually enabling
- `-b, --batch-size`: Number of APIs to enable at once (default: 10)
- `-h, --help`: Show help message

### Features

- Checks for gcloud installation and authentication
- Verifies project exists and you have access
- Shows current state (enabled vs available services)
- Processes APIs in configurable batches to avoid rate limiting
- Provides detailed progress updates
- Handles failures gracefully (tries one-by-one if batch fails)
- Shows comprehensive summary at the end
- Supports dry-run mode for safety

### Notes

- Enabling all APIs may have billing implications
- Some APIs require additional setup after enabling
- The script includes a confirmation prompt before making changes
- Failed services are reported at the end for manual review
