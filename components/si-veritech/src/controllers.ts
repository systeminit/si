import { inferProperties } from "./controllers/inferProperties";
import { checkQualifications } from "./controllers/checkQualifications";
import { loadWorkflows } from "./controllers/loadWorkflows";
import { runCommand } from "./controllers/runCommand";
import { syncResource } from "./controllers/syncResource";

const controller = {
  inferProperties,
  checkQualifications,
  loadWorkflows,
  runCommand,
  syncResource,
};

export default controller;
