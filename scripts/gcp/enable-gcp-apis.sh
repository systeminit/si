#!/usr/bin/env bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
DRY_RUN=false
BATCH_SIZE=10
PROJECT_ID=""

# Usage information
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Enable all available GCP APIs for a project.

OPTIONS:
    -p, --project       GCP Project ID (optional, uses gcloud configured project if not provided)
    -d, --dry-run       Show what would be enabled without actually enabling
    -b, --batch-size    Number of APIs to enable at once (default: 10)
    -h, --help          Show this help message

EXAMPLES:
    # Use configured project
    $0
    $0 --dry-run

    # Or specify project explicitly
    $0 --project my-project-id
    $0 --project my-project-id --dry-run
    $0 --project my-project-id --batch-size 20

NOTES:
    - Requires gcloud CLI to be installed and authenticated
    - Enabling all APIs may have billing implications
    - Some APIs may require additional setup after enabling

EOF
    exit 1
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--project)
            PROJECT_ID="$2"
            shift 2
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -b|--batch-size)
            BATCH_SIZE="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            ;;
    esac
done

# Check if gcloud is installed
if ! command -v gcloud &> /dev/null; then
    echo -e "${RED}Error: gcloud CLI is not installed${NC}"
    echo "Install from: https://cloud.google.com/sdk/docs/install"
    exit 1
fi

# Check if project ID is provided, otherwise use configured project
if [ -z "$PROJECT_ID" ]; then
    PROJECT_ID=$(gcloud config get-value project 2>/dev/null || echo "")
    if [ -z "$PROJECT_ID" ] || [ "$PROJECT_ID" = "(unset)" ]; then
        echo -e "${RED}Error: No project ID provided and no project configured${NC}"
        echo "Either:"
        echo "  1. Pass project ID: $0 --project PROJECT_ID"
        echo "  2. Set default project: gcloud config set project PROJECT_ID"
        exit 1
    fi
    echo -e "${YELLOW}Using configured project: $PROJECT_ID${NC}"
fi

# Check if user is authenticated
if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" &> /dev/null; then
    echo -e "${RED}Error: Not authenticated with gcloud${NC}"
    echo "Run: gcloud auth login"
    exit 1
fi

# Verify project exists
if ! gcloud projects describe "$PROJECT_ID" &> /dev/null; then
    echo -e "${RED}Error: Project '$PROJECT_ID' not found or you don't have access${NC}"
    exit 1
fi

echo -e "${GREEN}Project: $PROJECT_ID${NC}"
echo ""

# Get all available services
echo "Fetching all available GCP services..."
AVAILABLE_SERVICES=$(gcloud services list --available --project="$PROJECT_ID" --format="value(config.name)" 2>/dev/null)

if [ -z "$AVAILABLE_SERVICES" ]; then
    echo -e "${RED}Error: Could not fetch available services${NC}"
    exit 1
fi

TOTAL_SERVICES=$(echo "$AVAILABLE_SERVICES" | wc -l | tr -d ' ')
echo -e "${GREEN}Found $TOTAL_SERVICES available services${NC}"
echo ""

# Get currently enabled services
echo "Fetching currently enabled services..."
ENABLED_SERVICES=$(gcloud services list --enabled --project="$PROJECT_ID" --format="value(config.name)" 2>/dev/null || echo "")

if [ -n "$ENABLED_SERVICES" ]; then
    ENABLED_COUNT=$(echo "$ENABLED_SERVICES" | wc -l | tr -d ' ')
    echo -e "${GREEN}Currently enabled: $ENABLED_COUNT services${NC}"
else
    ENABLED_COUNT=0
    echo -e "${YELLOW}Currently enabled: 0 services${NC}"
fi

# Find services that need to be enabled
SERVICES_TO_ENABLE=""
while IFS= read -r service; do
    if [ -n "$ENABLED_SERVICES" ]; then
        if ! echo "$ENABLED_SERVICES" | grep -q "^$service$"; then
            SERVICES_TO_ENABLE="$SERVICES_TO_ENABLE$service"$'\n'
        fi
    else
        SERVICES_TO_ENABLE="$SERVICES_TO_ENABLE$service"$'\n'
    fi
done <<< "$AVAILABLE_SERVICES"

# Remove trailing newline and count
SERVICES_TO_ENABLE=$(echo "$SERVICES_TO_ENABLE" | sed '/^$/d')
TO_ENABLE_COUNT=$(echo "$SERVICES_TO_ENABLE" | wc -l | tr -d ' ')

if [ "$TO_ENABLE_COUNT" -eq 0 ]; then
    echo -e "${GREEN}All services are already enabled!${NC}"
    exit 0
fi

echo -e "${YELLOW}Services to enable: $TO_ENABLE_COUNT${NC}"
echo ""

# Show warning and ask for confirmation
if [ "$DRY_RUN" = false ]; then
    echo -e "${YELLOW}⚠️  WARNING ⚠️${NC}"
    echo "Enabling all APIs may:"
    echo "  - Incur additional costs"
    echo "  - Create quota allocations"
    echo "  - Take several minutes to complete"
    echo ""
    read -p "Do you want to proceed? (yes/no): " -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
        echo "Aborted."
        exit 0
    fi
fi

# Enable services
echo "Starting to enable services..."
echo ""

SUCCESS_COUNT=0
FAILED_COUNT=0
FAILED_SERVICES=()

# Process in batches
BATCH_SERVICES=()
CURRENT_BATCH=0

while IFS= read -r service; do
    BATCH_SERVICES+=("$service")
    CURRENT_BATCH=$((CURRENT_BATCH + 1))

    if [ "$CURRENT_BATCH" -eq "$BATCH_SIZE" ] || [ "$SUCCESS_COUNT" -eq "$((TO_ENABLE_COUNT - 1))" ]; then
        if [ "$DRY_RUN" = true ]; then
            echo -e "${YELLOW}[DRY RUN] Would enable:${NC}"
            printf '%s\n' "${BATCH_SERVICES[@]}"
            SUCCESS_COUNT=$((SUCCESS_COUNT + ${#BATCH_SERVICES[@]}))
        else
            echo "Enabling batch of ${#BATCH_SERVICES[@]} services..."
            if gcloud services enable "${BATCH_SERVICES[@]}" --project="$PROJECT_ID" 2>/dev/null; then
                SUCCESS_COUNT=$((SUCCESS_COUNT + ${#BATCH_SERVICES[@]}))
                echo -e "${GREEN}✓ Enabled ${#BATCH_SERVICES[@]} services ($SUCCESS_COUNT/$TO_ENABLE_COUNT)${NC}"
            else
                # If batch fails, try one by one
                for svc in "${BATCH_SERVICES[@]}"; do
                    if gcloud services enable "$svc" --project="$PROJECT_ID" 2>/dev/null; then
                        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
                        echo -e "${GREEN}✓ Enabled: $svc ($SUCCESS_COUNT/$TO_ENABLE_COUNT)${NC}"
                    else
                        FAILED_COUNT=$((FAILED_COUNT + 1))
                        FAILED_SERVICES+=("$svc")
                        echo -e "${RED}✗ Failed: $svc${NC}"
                    fi
                done
            fi
        fi

        # Reset batch
        BATCH_SERVICES=()
        CURRENT_BATCH=0

        # Small delay to avoid rate limiting
        if [ "$DRY_RUN" = false ]; then
            sleep 1
        fi
    fi
done <<< "$SERVICES_TO_ENABLE"

# Summary
echo ""
echo "============================================"
echo -e "${GREEN}Summary${NC}"
echo "============================================"
echo "Total available services: $TOTAL_SERVICES"
echo "Previously enabled: $ENABLED_COUNT"
echo "Newly enabled: $SUCCESS_COUNT"

if [ "$FAILED_COUNT" -gt 0 ]; then
    echo -e "${RED}Failed: $FAILED_COUNT${NC}"
    echo ""
    echo "Failed services:"
    printf '%s\n' "${FAILED_SERVICES[@]}"
fi

if [ "$DRY_RUN" = true ]; then
    echo ""
    echo -e "${YELLOW}This was a dry run. No services were actually enabled.${NC}"
    echo "Run without --dry-run to enable services."
fi

echo "============================================"

exit 0
