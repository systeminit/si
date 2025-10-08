import { run } from "./src/cli.ts";

// hi

if (import.meta.main) {
  await run();
}
