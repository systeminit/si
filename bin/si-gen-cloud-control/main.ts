import { run } from "./src/cli.ts";

if (import.meta.main) {
  await run();
}
