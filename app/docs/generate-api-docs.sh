#!/bin/bash
set -euo pipefail

# Get the repository root
REPO_ROOT=$(buck2 root)

# Build the OpenAPI spec
echo "Building OpenAPI spec..."
buck2 build "//bin/openapi-extractor:generate_api_spec"

# Get the output path of the OpenAPI spec
OPENAPI_SPEC=$(buck2 build "//bin/openapi-extractor:generate_api_spec" --show-full-output | grep -v "Action" | awk '{print $2}')

echo "OpenAPI spec located at: $OPENAPI_SPEC"

# Change to the docs directory
cd "$REPO_ROOT/app/docs"

# Run widdershins to generate the API documentation
echo "Generating API documentation..."
npx widdershins \
    --omitHeader \
    --search false \
    --language_tabs "typescript:TypeScript" \
    --summary \
    "$OPENAPI_SPEC" \
    -o src/reference/public-api.md \
    -u ./custom-templates/openapi3

echo "API documentation generated at src/reference/public-api.md"
