import { loadDatabase } from "../cfDb.ts";

export async function setupDatabase() {
  await loadDatabase();
}
