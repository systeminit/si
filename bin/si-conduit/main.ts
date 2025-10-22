#!/usr/bin/env -S deno run --allow-net --allow-env --allow-read --allow-write

import { start } from "./src/cli.ts";

if (import.meta.main) {
  await start();
}
