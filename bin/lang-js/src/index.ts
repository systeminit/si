#!/usr/bin/env tsc

import { Command } from "commander";
import fs from "fs";
import { executeRemoteFunction } from "./remote_function";

function main() {
  const program = new Command();
  program.version("0.0.1");
  program.parse(process.argv);
  const options = program.opts();
  const requestJson = fs.readFileSync(0, "utf8");
  const request = JSON.parse(requestJson);
  executeRemoteFunction(request);
}

main();
