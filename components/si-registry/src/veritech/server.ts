import express from "express";
import morgan from "morgan";
import chalk from "chalk";
import "@/loader";

import { intelligence } from "@/veritech/intelligence";

export const app = express();
app.use(express.json());
app.use(morgan("tiny"));

app.post("/intelligence", intelligence);

export function start(port: number): void {
  app.listen(port, () => {
    console.log(`Starting ${chalk.cyanBright("veritech")} on ${port}`);
  });
}
