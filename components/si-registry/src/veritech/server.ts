import express from "express";
import morgan from "morgan";
import chalk from "chalk";
import expressWs from "express-ws";
import "@/loader";
import "@/veritech/components/application";
import "@/veritech/components/awsIamJsonPolicy";
import "@/veritech/components/awsEks";
import "@/veritech/components/dockerImage";
import "@/veritech/components/kubernetesCluster";
import "@/veritech/components/kubernetesDeployment";
import "@/veritech/components/kubernetesNamespace";
import "@/veritech/components/kubernetesSecret";
import "@/veritech/components/kubernetesService";
import "@/veritech/components/service";
import "@/veritech/components/helmRepo";
import "@/veritech/components/helmChart";
import "@/veritech/components/helmRelease";

import { registry } from "@/registry";

import {
  calculateProperties,
  calculateConfigures,
  applyOp,
  action,
  syncResource,
} from "@/veritech/intelligence";

export const app = expressWs(express()).app;
app.use(express.json());
app.use(morgan("tiny"));

app.post("/calculateProperties", calculateProperties);
app.post("/calculateConfigures", calculateConfigures);
app.post("/applyOp", applyOp);
//app.post("/action", action);
//app.post("/syncResource", syncResource);
app.ws("/ws/action", (ws, _req) => {
  ws.on("message", function(msg: string) {
    action(ws, msg);
  });
});

app.ws("/ws/syncResource", (ws, _req) => {
  ws.on("message", function(msg: string) {
    syncResource(ws, msg);
  });
});

export function start(port: number): void {
  registry.serialize();
  const server = app.listen(port, () => {
    console.log(`Starting ${chalk.cyanBright("veritech")} on ${port}`);
  });
  // This is probably way, way too high. But still!
  server.keepAliveTimeout = 600000;
  server.headersTimeout = 601000;
}
