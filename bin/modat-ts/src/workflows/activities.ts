import { proxyActivities } from "@temporalio/workflow";

import type { createActivities } from "../activities";

export const activities = proxyActivities<ReturnType<typeof createActivities>>({
  startToCloseTimeout: "1 minute",
});
