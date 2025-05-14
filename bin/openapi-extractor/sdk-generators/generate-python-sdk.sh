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

  echo "Generating Python SDK from $SPEC_PATH to $OUTPUT_DIR..."

  cat > config.json << EOL
{
  "packageName": "system_initiative_api_client",
  "projectName": "system_initiative_api_client",
  "packageVersion": "1.0.0",
  "hideGenerationTimestamp": true,
  "gitUserId": "",
  "gitRepoId": "",
  "disableHtmlEscaping": true,
  "legacyDiscriminatorBehavior": false
}
EOL

  openapi-generator-cli generate \
      -i "$SPEC_PATH" \
      -g python \
      -o "$OUTPUT_DIR" \
      -c config.json \
      --skip-operation-example \
      --skip-validate-spec

  rm config.json

  find "$OUTPUT_DIR" -name ".openapi-generator" -type d -exec rm -rf {} +
  find "$OUTPUT_DIR" -name "docs" -type d -exec rm -rf {} +
  find "$OUTPUT_DIR" -name ".github" -type d -exec rm -rf {} +
  find "$OUTPUT_DIR" -name ".gitignore"  -type f -delete
  find "$OUTPUT_DIR" -name ".travis.yml" -type f -delete
  find "$OUTPUT_DIR" -name ".gitlab-ci.yml" -type f -delete
  find "$OUTPUT_DIR" -name "git_push.sh" -type f -delete
  find "$OUTPUT_DIR" -name ".openapi-generator-ignore" -type f -delete

  echo "SDK generation successful! SDK files available at: $OUTPUT_DIR"
  echo "To install the SDK, run:"
  echo "  cd $OUTPUT_DIR"
  echo "  pip install -e ."
  echo "Generation complete"
  exit
}

echo "Generating Python SDK"
generate_sdk

exit 0
