import axios from "npm:axios";
import { apiConfig, WORKSPACE_ID } from "./si_client.ts";
import { logger } from "./logger.ts";

export async function setAiAgentUserFlag() {
  // Skip if apiConfig is not available (e.g., in test mode or no token)
  if (!apiConfig || !WORKSPACE_ID) {
    return;
  }

  const url =
    `${apiConfig.basePath}/v1/w/${WORKSPACE_ID}/user/set_ai_agent_executed`;

  try {
    await axios.post(url, {}, apiConfig.baseOptions);
  } catch (error) {
    logger.error("Error setting AI agent flag:", error);
  }
}
