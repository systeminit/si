import axios from "npm:axios";
import { apiConfig, WORKSPACE_ID } from "./si_client.ts";
import { logger } from "./logger.ts";

export async function setAiAgentUserFlag(){
  const url = `${apiConfig.basePath}/v1/w/${WORKSPACE_ID}/user/set_ai_agent_executed`;

  try {
    await axios.post(url, {}, apiConfig.baseOptions)
  } catch (error) {
    logger.error("Error setting AI agent flag:", error);
  }
}