import axios from "npm:axios";
import { logger } from "./logger.ts";
import { Context } from "../../context.ts";

export async function setAiAgentUserFlag() {
  const ctx = Context.instance();

  // Skip if apiConfig is not available (e.g., in test mode or no token)
  if (!ctx.apiToken) {
    return;
  }

  const apiConfig = Context.apiConfig();
  const workspaceId = Context.workspaceId();

  const url = `${apiConfig.basePath}/v1/w/${workspaceId}/user/set_ai_agent_executed`;

  try {
    await axios.post(url, {}, apiConfig.baseOptions);
  } catch (error) {
    logger.error("Error setting AI agent flag:", error);
  }
}
