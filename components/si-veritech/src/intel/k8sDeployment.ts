import Debug from "debug";
const debug = Debug("veritech:controllers:intel:k8sDeployment");

import { RunCommandCallbacks } from "../controllers/runCommand";

export const runCommands: RunCommandCallbacks = {
  apply: async function (ctx, req, ws) {
    const names = [];
    for (const contextEntry of req.selection.context) {
      debug("entry", contextEntry.entity.name);
      names.push(contextEntry.entity.name);
    }
    await ctx.execStream(ws, "echo", names);
  },
};

export default { runCommands };
