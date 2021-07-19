import { Context } from "koa";
import api from "@opentelemetry/api";

import Debug from "debug";
const debug = Debug("veritech:controllers:loadWorkflows");

import { workflows } from "si-registry";

export function loadWorkflows(ctx: Context): void {
  const span = api.trace.getSpan(api.context.active());
  span.updateName("veritech.loadworkflows");
  debug("request body: %O", ctx.request.body);
  ctx.response.status = 200;
  const response = { workflows: Object.values(workflows) };
  //const response = { workflows: [] };

  // output json on the console with line numbers, because....debugging
  const str = JSON.stringify(response, null, 2);
  str.split("\n").forEach((line, index) => {
    debug("body(%s)%s", (index + 1).toString().padStart(3, "0"), line);
  });

  ctx.response.body = str;
}
