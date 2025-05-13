#!/usr/bin/env bash
set -e

if ! command -v openapi-generator-cli &> /dev/null; then
    echo "Error: openapi-generator-cli not found."
    exit 1
fi

SPEC_PATH=$1
OUTPUT_DIR=$2

if [ ! -f "$SPEC_PATH" ]; then
    echo "Error: OpenAPI specification file not found at $SPEC_PATH"
    exit 1
fi

generate_sdk() {
  mkdir -p "$OUTPUT_DIR"
  echo "Generating TypeScript SDK from $SPEC_PATH to $OUTPUT_DIR..."

  cat > config.json << EOL
{
  "npmName": "system-initiative-api-client",
  "npmVersion": "1.0.0",
  "supportsES6": true,
  "modelPropertyNaming": "camelCase",
  "enumPropertyNaming": "UPPERCASE",
  "withInterfaces": true,
  "typescriptThreePlus": true,
  "useSingleRequestParameter": true,
  "hideGenerationTimestamp": true,
  "stringEnums": true
}
EOL

  openapi-generator-cli generate \
      -i "$SPEC_PATH" \
      -g typescript-axios \
      -o "$OUTPUT_DIR" \
      -c config.json \
      --skip-validate-spec

  rm config.json

  find "$OUTPUT_DIR" -name ".openapi-generator" -type d -exec rm -rf {} +
  find "$OUTPUT_DIR" -name "docs" -type d -exec rm -rf {} +
  find "$OUTPUT_DIR" -name ".github" -type d -exec rm -rf {} +
  find "$OUTPUT_DIR" -name ".gitignore" -type f -delete
  find "$OUTPUT_DIR" -name ".travis.yml" -type f -delete
  find "$OUTPUT_DIR" -name ".gitlab-ci.yml" -type f -delete
  find "$OUTPUT_DIR" -name "git_push.sh" -type f -delete
  find "$OUTPUT_DIR" -name ".openapi-generator-ignore" -type f -delete

  echo "SDK generation successful! SDK files available at: $OUTPUT_DIR"
  echo "To install the SDK, run:"
  echo "  cd $OUTPUT_DIR"
  echo "  npm install"
  echo "  npm run build"
  echo ""
  echo "To use the SDK in your project:"
  echo "  npm pack  # Creates a tarball"
  echo "  # Then in your project:"
  echo "  npm install path/to/system-initiative-api-client-1.0.0.tgz"
}

echo "Generating TypeScript SDK"
generate_sdk

exit 0
