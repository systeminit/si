#!/usr/bin/env tsc

import fs from "fs";
import { Command } from "commander";
import Debug from "debug";
import { failureExecution, FunctionKind, function_kinds } from "./function";
import { executeResolverFunction } from "./resolver_function";
import { makeConsole } from "./sandbox/console";
import { executeQualificationCheck } from "./qualification_check";

const debug = Debug("langJs");

function main() {
  let kind;

  const program = new Command();
  program
    .version("0.0.1")
    .argument(
      "<kind>",
      `kind of function to be executed [values: ${function_kinds().join(", ")}]`
    )
    .action((kind_arg) => {
      if (Object.values(FunctionKind).includes(kind_arg)) {
        kind = kind_arg;
      } else {
        console.error(`Unsupported function kind: '${kind_arg}'`);
        process.exit(1);
      }
    })
    .parse(process.argv);

  let executionId;
  // We don't have the executionId yet, so this field will be empty
  let error = makeConsole("").error;

  try {
    const requestJson = fs.readFileSync(0, "utf8");
    const request = JSON.parse(requestJson);
    debug({ request });
    if (request.executionId) {
      executionId = request.executionId;
    } else {
      throw Error("Request must have executionId field");
    }
    // Now we have the executionId, so update our console.error() impl
    error = makeConsole(executionId).error;

    switch (kind) {
      case FunctionKind.QualificationCheck:
        executeQualificationCheck(request);
        break;
      case FunctionKind.ResolverFunction:
        executeResolverFunction(request);
        break;
      default:
        throw Error(`Unknown Kind variant: ${kind}`);
    }
  } catch (err) {
    debug(err);
    error("StackTrace", err.stack);
    console.log(JSON.stringify(failureExecution(err, executionId)));
    process.exit(1);
  }
}

main();
