import { inferProperties } from "./controllers/inferProperties";
import { checkQualifications } from "./controllers/checkQualifications";
import { loadWorkflows } from "./controllers/loadWorkflows";
import { runCommand } from "./controllers/runCommand";
import { syncResource } from "./controllers/syncResource";
import { discover } from "./controllers/discover";

const controller = {
  inferProperties,
  checkQualifications,
  loadWorkflows,
  runCommand,
  syncResource,
  discover,
};

export default controller;
