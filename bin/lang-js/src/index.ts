#!/usr/bin/env node

import process from "node:process";
import { Command } from "jsr:@cliffy/command@^1.0.0-rc.7";
import { makeConsole } from "./sandbox/console.ts";
import { Request } from "./request.ts";
import {
  executeFunction,
  failureExecution,
  FunctionKind,
  functionKinds,
} from "./function.ts";
import { Debug } from "./debug.ts";

// This is the default timeout for a function, in seconds.
const defaultTimeout = 1800;

const debug = Debug("langJs");

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
  const { options } = await new Command()
    .name("lang-js")
    .version("0.0.1")
    .option(
      "--timeout <seconds:number>",
      `timeout for a function execution in seconds (default: ${defaultTimeout})`,
      { default: defaultTimeout },
    )
    .parse(Deno.args);

  let timeout: number = defaultTimeout;
  if (options.timeout) {
    timeout = options.timeout;
    if (Number.isNaN(timeout) || !Number.isFinite(timeout)) {
      console.error(
        `Unsupported value for timeout (expected 'number'): ${options.timeout}`,
      );
      Deno.exit(1);
    }
  }

  let executionId = "<unset>";
  // We don't have the executionId yet, so this field will be empty
  let errorFn = makeConsole(executionId).error;

  try {
    const decoder = new TextDecoder();
    let requestJson = "";
    for await (const chunk of Deno.stdin.readable) {
      requestJson += decoder.decode(chunk);
    }
    debug({ request: requestJson });
    const request: Request = JSON.parse(requestJson);

    if (request.executionId) {
      executionId = request.executionId;
    } else {
      throw Error("Request must have executionId field");
    }

    if (!request.kind) {
      throw Error("Request must have a kind field");
    }

    debug({ request });

    // Now we have the executionId, so update our console.error() impl
    errorFn = makeConsole(executionId).error;

    // Async errors inside VM2 have to be caught here, they escape the try/catch wrapping the vm.run call
    process.on("uncaughtException", (err: Error) => {
      onError(errorFn, err, executionId);
    });

    await executeFunction(request, timeout);
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
  Deno.exit(0);
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
