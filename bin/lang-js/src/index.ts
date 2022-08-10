#!/usr/bin/env tsc

import fs from "fs";
import { Command } from "commander";
import Debug from "debug";
import { failureExecution, FunctionKind, function_kinds } from "./function";
import { makeConsole } from "./sandbox/console";
import { executeCodeGeneration } from "./code_generation";
import { executeQualificationCheck } from "./qualification_check";
import { executeResolverFunction } from "./resolver_function";
import { executeResourceSync } from "./resource_sync";
import { executeWorkflowResolve } from "./workflow_resolve";

const debug = Debug("langJs");

async function main() {
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
    debug({ request, data: JSON.stringify(request?.data) });
    if (request.executionId) {
      executionId = request.executionId;
    } else {
      throw Error("Request must have executionId field");
    }
    // Now we have the executionId, so update our console.error() impl
    error = makeConsole(executionId).error;

    switch (kind) {
      case FunctionKind.CodeGeneration:
        executeCodeGeneration(request);
        break;
      case FunctionKind.QualificationCheck:
        await executeQualificationCheck(request);
        break;
      case FunctionKind.ResolverFunction:
        executeResolverFunction(request);
        break;
      case FunctionKind.ResourceSync:
        executeResourceSync(request);
        break;
      case FunctionKind.WorkflowResolve:
        executeWorkflowResolve(request);
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

// interface Errorable {
//   name: string;
//   message: string;
//   stack?: string;
// }
//
// function isErrorable(err: unknown): err is Errorable {
//   return (
//     typeof err == "object" &&
//     err !== null &&
//     "name" in err &&
//     typeof (err as Record<string, unknown>).name === "string" &&
//     "message" in err &&
//     typeof (err as Record<string, unknown>).message === "string" &&
//     "stack" in err &&
//     typeof (err as Record<string, unknown>).stack === "string"
//   );
// }
//
// function toErrorable(maybeError: unknown): Errorable {
//   if (isErrorable(maybeError)) {
//     return maybeError;
//   }
//
//   try {
//     return new Error(JSON.stringify(maybeError));
//   } catch {
//     return new Error(String(maybeError));
//   }
// }
//
// function failAndDie(e: unknown, executionId: unknown) {
//   const err = toErrorable(e);
//   debug(err);
//   error("StackTrace", err.stack);
//   console.log(JSON.stringify(failureExecution(err, executionId)));
//   process.exit(1);
// }

main();
