#!/usr/bin/env bash
set -e

if ! command -v openapi-generator-cli &>/dev/null; then
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

  # Download LICENSE file from System Initiative repo
  echo "Downloading LICENSE file..."
  curl -s "https://raw.githubusercontent.com/systeminit/si/main/LICENSE" >"$OUTPUT_DIR/LICENSE"

  cat >config.json <<EOL
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
  "stringEnums": true,
  "packageName": "system-initiative-api-client",
  "packageVersion": "1.0.0",
  "packageUrl": "https://github.com/systeminit/si",
  "author": "System Initiative",
  "authorEmail": "support@systeminit.com",
  "developerName": "System Initiative",
  "developerEmail": "info@systeminit.com",
  "developerOrganization": "System Initiative",
  "developerOrganizationUrl": "https://systeminit.com",
  "artifactDescription": "TypeScript/JavaScript SDK for the System Initiative Public API",
  "artifactUrl": "https://github.com/systeminit/si",
  "licenseId": "Apache-2.0",
  "licenseName": "Apache License 2.0",
  "licenseUrl": "https://www.apache.org/licenses/LICENSE-2.0.html"
}
EOL

  # Extract package data from config.json before it's removed
  PACKAGE_NAME=$(grep -o '"packageName": "[^"]*' config.json | cut -d'"' -f4)
  PACKAGE_VERSION=$(grep -o '"packageVersion": "[^"]*' config.json | cut -d'"' -f4)
  NPM_NAME=$(grep -o '"npmName": "[^"]*' config.json | cut -d'"' -f4)
  PACKAGE_URL=$(grep -o '"packageUrl": "[^"]*' config.json | cut -d'"' -f4)
  AUTHOR=$(grep -o '"author": "[^"]*' config.json | cut -d'"' -f4)
  AUTHOR_EMAIL=$(grep -o '"authorEmail": "[^"]*' config.json | cut -d'"' -f4)
  ARTIFACT_DESCRIPTION=$(grep -o '"artifactDescription": "[^"]*' config.json | cut -d'"' -f4)
  LICENSE_NAME=$(grep -o '"licenseName": "[^"]*' config.json | cut -d'"' -f4)
  LICENSE_ID=$(grep -o '"licenseId": "[^"]*' config.json | cut -d'"' -f4)
  LICENSE_URL=$(grep -o '"licenseUrl": "[^"]*' config.json | cut -d'"' -f4)
  DEVELOPER_NAME=$(grep -o '"developerName": "[^"]*' config.json | cut -d'"' -f4)
  DEVELOPER_EMAIL=$(grep -o '"developerEmail": "[^"]*' config.json | cut -d'"' -f4)
  DEVELOPER_ORG=$(grep -o '"developerOrganization": "[^"]*' config.json | cut -d'"' -f4)
  DEVELOPER_ORG_URL=$(grep -o '"developerOrganizationUrl": "[^"]*' config.json | cut -d'"' -f4)

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

  # Update package.json with all metadata
  PACKAGE_JSON="$OUTPUT_DIR/package.json"
  if [ -f "$PACKAGE_JSON" ]; then
    echo "Updating package.json with all metadata..."

    TMP_PACKAGE_JSON="$OUTPUT_DIR/package.json.new"

    cat >"$TMP_PACKAGE_JSON" <<EOF
{
  "name": "${NPM_NAME}",
  "version": "${PACKAGE_VERSION}",
  "description": "${ARTIFACT_DESCRIPTION}",
  "author": "${AUTHOR} <${AUTHOR_EMAIL}>",
  "repository": {
    "type": "git",
    "url": "${PACKAGE_URL}"
  },
  "homepage": "https://systeminit.com",
  "bugs": {
    "url": "${PACKAGE_URL}/issues"
  },
  "keywords": [
    "axios",
    "typescript",
    "javascript",
    "openapi",
    "api-client",
    "system-initiative",
    "infrastructure-as-code"
  ],
  "license": "${LICENSE_ID}",
  "main": "./dist/cjs/index.js",
  "types": "./dist/cjs/index.d.ts",
  "module": "./dist/esm/index.js",
  "exports": {
    ".": {
      "import": "./dist/esm/index.js",
      "require": "./dist/cjs/index.js",
      "types": "./dist/cjs/index.d.ts"
    }
  },
  "sideEffects": false,
  "scripts": {
    "build": "tsc -p tsconfig.json && tsc -p tsconfig.esm.json",
    "prepare": "npm run build",
    "test": "echo \"No tests yet\""
  },
  "dependencies": {
    "axios": "^1.6.1"
  },
  "devDependencies": {
    "@types/node": "^18.0.0",
    "typescript": "^5.0.0"
  },
  "engines": {
    "node": ">=18.0.0"
  },
  "publishConfig": {
    "access": "public"
  },
  "files": [
    "dist/cjs",
    "README.md",
    "LICENSE",
    "package.json"
  ]
}
EOF

    # Replace original with our modified version
    mv "$TMP_PACKAGE_JSON" "$PACKAGE_JSON"
    echo "package.json updated with all metadata"
  fi

  NPMIGNORE="$OUTPUT_DIR/.npmignore"
  echo "Creating .npmignore file for NPM publishing ..."

  cat >"$NPMIGNORE" <<EOF

# Ignore TypeScript config files
tsconfig.json
tsconfig.esm.json

# Ignore environment configuration files
*.env

# Ignore git and other version control files
.git/
.gitignore
EOF
  echo "Created .npmignore file for NPM publishing"

  # Create deno.json for JSR publishing
  DENO_JSON="$OUTPUT_DIR/deno.json"
  echo "Creating deno.json for JSR publishing..."

  cat >"$DENO_JSON" <<EOF
{
  "name": "@systeminit/api-client",
  "version": "${PACKAGE_VERSION}",
  "exports": {
    ".": "./index.ts"
  },
  "lint": {
    "rules": {
      "exclude": [
        "missing-explicit-return-type",
        "no-explicit-any",
        "camelcase"
      ]
    }
  },
  "publish": {
    "exclude": [
      ".git",
      "node_modules",
      "dist",
      ".openapi-generator*",
      "package*.json",
      "tsconfig.json",
      ".npmignore"
    ]
  },
  "fmt": {
    "lineWidth": 100,
    "indentWidth": 2,
    "singleQuote": true,
    "semiColons": true
  },
  "workspace": []
}
EOF
  echo "Created deno.json for JSR publishing"

  # Update tsconfig.json to improve TypeScript configuration
  TSCONFIG_JSON="$OUTPUT_DIR/tsconfig.json"
  if [ -f "$TSCONFIG_JSON" ]; then
    echo "Updating tsconfig.json with modern settings..."

    TMP_TSCONFIG_JSON="$OUTPUT_DIR/tsconfig.json.new"

    cat >"$TMP_TSCONFIG_JSON" <<EOF
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "CommonJS",
    "moduleResolution": "node",
    "declaration": true,
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "outDir": "dist/cjs",
    "rootDir": ".",
    "lib": ["ES2020", "DOM"],
    "typeRoots": ["node_modules/@types"]
  },
  "include": ["*.ts"],
  "exclude": ["dist", "node_modules"]
}
EOF

    # Replace original with our modified version
    mv "$TMP_TSCONFIG_JSON" "$TSCONFIG_JSON"
    echo "tsconfig.json updated with modern settings"
  fi

  # Update ESM tsconfig
  TSCONFIG_ESM_JSON="$OUTPUT_DIR/tsconfig.esm.json"
  if [ -f "$TSCONFIG_ESM_JSON" ]; then
    echo "Updating tsconfig.esm.json for ESM output..."

    TMP_TSCONFIG_ESM_JSON="$OUTPUT_DIR/tsconfig.esm.json.new"

    cat >"$TMP_TSCONFIG_ESM_JSON" <<EOF
{
  "extends": "./tsconfig.json",
  "compilerOptions": {
    "module": "ESNext",
    "outDir": "dist/esm"
  },
  "include": ["*.ts"]
}
EOF

    # Replace original with our modified version
    mv "$TMP_TSCONFIG_ESM_JSON" "$TSCONFIG_ESM_JSON"
    echo "tsconfig.esm.json updated for ESM output"
  fi

  # Update README.md with better documentation
  README_MD="$OUTPUT_DIR/README.md"
  if [ -f "$README_MD" ]; then
    echo "Updating README.md with improved content..."
    cat >"$README_MD" <<EOF
# ${NPM_NAME}

${ARTIFACT_DESCRIPTION}

## Installation

### npm

\`\`\`
$ npm install ${NPM_NAME}
\`\`\`

### Yarn

\`\`\`
$ yarn add ${NPM_NAME}
\`\`\`

### Deno / JSR

\`\`\`
import { Configuration, ChangeSetsApi } from "@systeminit/api-client";
\`\`\`

## Requirements

- Node.js >=18.0.0 or Deno

## Usage

### Node.js (CommonJS)

\`\`\`
const { Configuration, ChangeSetsApi } = require('${NPM_NAME}');

// Configure API key authorization
const apiToken = process.env.SI_API_TOKEN;
const config = new Configuration({
  basePath: 'https://api.systeminit.com',
  headers: {
    Authorization: \`Bearer \${apiToken}\`,
  }
});

const workspaceId = process.env.SI_WORKSPACE_ID;
const changeSetsApi = new ChangeSetsApi(config);

// Example API client usage
async function listChangeSets() {
  try {
    const response = await changeSetsApi.listChangeSets(workspaceId);
    console.log(JSON.stringify(response.data, null, 2));
  } catch (error) {
    console.error('Error listing change sets:', error);
  }
}

listChangeSets();
\`\`\`

### Node.js (ESM)

\`\`\`
import { Configuration, ChangeSetsApi } from '${NPM_NAME}';

// Configuration and usage same as above
\`\`\`

### Deno / JSR

\`\`\`
import { Configuration, ChangeSetsApi } from "@systeminit/api-client";

// Configure API key authorization
const apiToken = Deno.env.get("SI_API_TOKEN");
const config = new Configuration({
  basePath: "https://api.systeminit.com",
  headers: {
    Authorization: \`Bearer \${apiToken}\`,
  }
});

const workspaceId = Deno.env.get("SI_WORKSPACE_ID");
const changeSetsApi = new ChangeSetsApi(config);

// Example API client usage
try {
  const response = await changeSetsApi.listChangeSets(workspaceId);
  console.log(JSON.stringify(response.data, null, 2));
} catch (error) {
  console.error("Error listing change sets:", error);
}
\`\`\`

## Authentication

This API uses Bearer token authentication.

Make sure to include the token in the Authorization header as shown in the examples above.

## Documentation

For more details on the available endpoints and models, see the [System Initiative API documentation](https://docs.systeminit.com/reference/public-api).

## License

[${LICENSE_NAME}](${LICENSE_URL})

## Author Information

- **${AUTHOR}** - ${AUTHOR_EMAIL}
- **${DEVELOPER_NAME}** - ${DEVELOPER_EMAIL}
- **Organization**: ${DEVELOPER_ORG} - ${DEVELOPER_ORG_URL}

## Development

For development, clone this repository and install in development mode:

\`\`\`
git clone ${PACKAGE_URL}
cd generated-sdks/typescript
npm install
npm run build
\`\`\`

## Publishing

### To NPM

\`\`\`
cd generated-sdks/typescript
npm publish
\`\`\`

### To JSR (Deno)

\`\`\`
cd generated-sdks/typescript
deno publish
\`\`\`
EOF
    echo "README.md updated with improved content"
  fi

  echo "SDK generation successful! SDK files available at: $OUTPUT_DIR"
  echo "To install the SDK, run:"
  echo "  cd $OUTPUT_DIR"
  echo "  npm install"
  echo "  npm run build"
  echo ""
  echo "To publish to npm:"
  echo "  npm publish"
  echo ""
  echo "To publish to JSR (Deno):"
  echo "  deno publish"
}

echo "Generating TypeScript SDK"
generate_sdk

exit 0
