#!/usr/bin/env node

// placeholder that just includes the built files in dist
// we need this because the dist folder does not exist during pnpm install
// see https://github.com/vercel/turbo/discussions/446
require("./dist/index.js");
