# system-initiative-api-client

TypeScript/JavaScript SDK for the System Initiative Public API

## Installation

### npm

```
$ npm install system-initiative-api-client
```

### Yarn

```
$ yarn add system-initiative-api-client
```

### Deno / JSR

```
import { Configuration, ChangeSetsApi } from "@systeminit/api-client";
```

## Requirements

- Node.js >=18.0.0 or Deno

## Usage

### Node.js (CommonJS)

```
const { Configuration, ChangeSetsApi } = require('system-initiative-api-client');

// Configure API key authorization
const apiToken = process.env.SI_API_TOKEN;
const config = new Configuration({
  basePath: 'https://api.systeminit.com',
  headers: {
    Authorization: `Bearer ${apiToken}`,
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
```

### Node.js (ESM)

```
import { Configuration, ChangeSetsApi } from 'system-initiative-api-client';

// Configuration and usage same as above
```

### Deno / JSR

```
import { Configuration, ChangeSetsApi } from "@systeminit/api-client";

// Configure API key authorization
const apiToken = Deno.env.get("SI_API_TOKEN");
const config = new Configuration({
  basePath: "https://api.systeminit.com",
  headers: {
    Authorization: `Bearer ${apiToken}`,
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
```

## Authentication

This API uses Bearer token authentication.

Make sure to include the token in the Authorization header as shown in the examples above.

## Documentation

For more details on the available endpoints and models, see the [System Initiative API documentation](https://docs.systeminit.com/reference/public-api).

## License

[Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0.html)

## Author Information

- **System Initiative** - support@systeminit.com
- **System Initiative** - info@systeminit.com
- **Organization**: System Initiative - https://systeminit.com

## Development

For development, clone this repository and install in development mode:

```
git clone https://github.com/systeminit/si
cd generated-sdks/typescript
npm install
npm run build
```

## Publishing

### To NPM

```
cd generated-sdks/typescript
npm publish
```

### To JSR (Deno)

```
cd generated-sdks/typescript
deno publish
```
