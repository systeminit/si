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

// This is the default timeout for a function, in seconds.
const defaultTimeout = 1800;

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
    .option(
      "--timeout <seconds>",
      `timeout for a function execution in seconds (default: ${defaultTimeout})`,
    )
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

  const options = program.opts();

  let timeout = defaultTimeout;
  if (options.timeout) {
    timeout = parseInt(options.timeout);
    if (Number.isNaN(timeout) || !Number.isFinite(timeout)) {
      console.error(`Unsupported value for timeout (expected 'number'): ${options.timeout}`);
      process.exit(1);
    }
  }

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

    await executeFunction(kind, request, timeout);
  } catch (err) {
    onError(errorFn, err as Error, executionId);
  }

  // NOTE(nick): hey friends, Nick here. I won't implicate @jobelenus in this comment because I have a solid chance of
  // going off the rails, but I do need to give him attribution here. Thank you for pairing with me to find this.
  // Alright so here's the deal: whether I am employed at SI or not (or even contributing to SI or not), I need you to
  // ask yourself if you want to remove or change the this line of code. I'll tell you why. In hardening the system, we
  // plugged through configurable timeouts throughout the system: lang-js, cyclone-server, cyclone-client, etc. While
  // we shouldn't trust lang-js to clean itself up (parent process "cyclone" is more trustworthy for that ask), we do
  // want a world where timeouts work in lang-js. To get to this world, we create a "timeout" Promise and use
  // "Promise.race" to handle the race between the aforementioned Promise and the function execution Promise. That
  // upstream function creates a new Promise that is either resolved or rejected based on the first Promise returned.
  // The other Promise(s), in this case one, will not be "cancelled" in the same way a Rust future would be in a tokio
  // select block. They will run until the heat death of the universe... well until they're done, at least. That is why
  // we must self-destruct with an exit code of "0". It will kill the other Promise that is in-flight. This is sneaky
  // because lang-js exits with an exit code of "1" upon hitting an error, like a timeout, so all Promises are killed
  // when errors occur. I thought my timeout code was working... when it was only half working. You need to kill the
  // Promises on success too. Anyway, I'll stop yapping, but yeah. Just think REALLY hard if you want to remove or
  // change this line of code.
  process.exit(0);
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
