import { activities } from "./activities";
import { logger } from "../logger";

/** A workflow that simply calls an activity */
export async function example(name: string): Promise<string> {
  logger.info(`Greeting user ${name}`);

  return await activities.greet(name);
}
