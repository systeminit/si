#!/usr/bin/env node

import fs from "fs";
import { Command } from "commander";
import { makeConsole } from "./sandbox/console";
import { Request } from "./request";
import {
  executeFunction,
  failureExecution,
  FunctionKind,
  functionKinds,
} from "./function";
import { Debug } from "./debug";

const debug = Debug("langJs");
const STDIN_FD = 0;

function onError(
  errorFn: (...args: unknown[]) => void,
  err: Error,
  executionId: string,
) {
  debug(err);
  errorFn("StackTrace", err.stack);
  console.log(JSON.stringify(failureExecution(err, executionId)));
  process.exit(1);
}

async function main() {
  let kind: FunctionKind | undefined;

  const program = new Command();
  program
    .version("0.0.1")
    .argument(
      "<kind>",
      `kind of function to be executed [values: ${functionKinds().join(", ")}]`,
    )
    .action((kind_arg) => {
      if (functionKinds().includes(kind_arg)) {
        kind = kind_arg;
      } else {
        console.error(`Unsupported function kind: '${kind_arg}'`);
        process.exit(1);
      }
    })
    .parse(process.argv);

  let executionId = "<unset>";
  // We don't have the executionId yet, so this field will be empty
  let errorFn = makeConsole(executionId).error;

  try {
    const requestJson = fs.readFileSync(STDIN_FD, "utf8");
    debug({ request: requestJson });
    const request: Request = JSON.parse(requestJson);
    if (request.executionId) {
      executionId = request.executionId;
    } else {
      throw Error("Request must have executionId field");
    }
    // Now we have the executionId, so update our console.error() impl
    errorFn = makeConsole(executionId).error;

    // Async errors inside VM2 have to be caught here, they escape the try/catch wrapping the vm.run call
    process.on("uncaughtException", (err) => {
      onError(errorFn, err, executionId);
    });

    if (kind === undefined) {
      throw Error(`Kind is undefined`);
    }

    await executeFunction(kind, request);
  } catch (err) {
    onError(errorFn, err as Error, executionId);
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
main().catch(() => process.exit(1));
