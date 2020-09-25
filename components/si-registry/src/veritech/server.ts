import express from "express";
import morgan from "morgan";
import chalk from "chalk";
import "@/loader";

import {
  calculateProperties,
  calculateConfigures,
  applyOp,
} from "@/veritech/intelligence";

export const app = express();
app.use(express.json());
app.use(morgan("tiny"));

app.post("/calculateProperties", calculateProperties);
app.post("/calculateConfigures", calculateConfigures);
app.post("/applyOp", applyOp);

export function start(port: number): void {
  const server = app.listen(port, () => {
    console.log(`Starting ${chalk.cyanBright("veritech")} on ${port}`);
  });
  // This is probably way, way too high. But still!
  server.keepAliveTimeout = 600000;
  server.headersTimeout = 601000;
}
