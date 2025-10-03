#!/usr/bin/env -S deno run --allow-net --allow-env

import { run } from "./src/cli.ts";

if (import.meta.main) {
  await run();
}
