import { defineConfig } from 'vitest/config';
import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

export default defineConfig({
  resolve: {
    conditions: ['require', 'node'],
    alias: {
      'system-initiative-api-client': resolve(__dirname, 'node_modules/system-initiative-api-client/dist/cjs/index.js'),
    },
  },
  test: {
    globals: true,
    environment: 'node',
    testTimeout: 30000,
    hookTimeout: 30000,
    env: {
      SI_WORKSPACE_ID: process.env.SI_WORKSPACE_ID || '',
      SI_API_TOKEN: process.env.SI_API_TOKEN || '',
      SI_API_BASE_PATH: process.env.SI_API_BASE_PATH || 'https://api.systeminit.com',
    },
  },
});
