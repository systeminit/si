#!/usr/bin/env -S deno run --allow-net --allow-env

import { start } from "./src/cli2.ts";

if (import.meta.main) {
  await start();
}
