#!/bin/bash
set -euo pipefail

REPO_ROOT=$(buck2 root)
ACTUAL_SCRIPT_DIR="$REPO_ROOT/generated-sdks"

update_sdk() {
    local sdk_type=$1
    local target_name="generate_${sdk_type}_sdk"

    echo "Generating ${sdk_type} SDK..."

    buck2 build "//bin/openapi-extractor:${target_name}"
    SDK_PATH=$(buck2 build "//bin/openapi-extractor:${target_name}" --show-full-output | grep -v "Action" | awk '{print $2}')
    TARGET_DIR="$ACTUAL_SCRIPT_DIR/${sdk_type}"

    mkdir -p "$TARGET_DIR"
    find "$TARGET_DIR" -mindepth 1 -not -name '.gitkeep' -delete 2>/dev/null || true

    if [ -d "$SDK_PATH" ]; then
        cd "$SDK_PATH"
        find . -type f -exec cp --parents -v {} "$TARGET_DIR" \; 2>/dev/null || true
        find . -type d -exec mkdir -p "$TARGET_DIR/{}" \; 2>/dev/null || true
    else
        echo "Error: Expected directory not found at $SDK_PATH"
        exit 1
    fi

    echo "${sdk_type} SDK updated in $TARGET_DIR"
}

echo "Updating all SDKs..."
update_sdk "python"
update_sdk "typescript"
