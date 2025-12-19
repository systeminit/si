import { setTimeout } from "node:timers/promises";
import { setTimeout as setTimeoutCb } from "node:timers";
import { execa, type Result, type Options } from "npm:execa";

export interface WatchArgs {
  cmd: string;
  args?: readonly string[];
  execaOptions?: Options;
  retryMs?: number;
  maxRetryCount?: number;
  callback: (child: Result) => Promise<boolean>;
}

export interface WatchResult {
  result: SiExecResult;
  failed?: "deadlineExceeded" | "commandFailed";
}

export interface RetryOptions {
  maxAttempts?: number;
  baseDelay?: number;
  maxDelay?: number;
  jitter?: boolean;
  isRateLimitedFn?: (error: any) => boolean;
}

export interface LROPollOptions {
  url: string;
  headers: Record<string, string>;
  maxAttempts?: number;
  baseDelay?: number;
  maxDelay?: number;
  isCompleteFn: (response: Response, body: any) => boolean;
  isErrorFn?: (response: Response, body: any) => boolean;
  extractResultFn?: (response: Response, body: any) => any | Promise<any>;
}

export interface RetryResult<T> {
  result: T;
  attempts: number;
}

// import readline from "readline";
// import WebSocket from "ws";

export type SiExecResult = Result;

const defaultOptions: Options = {
  all: true,
  buffer: true,
  reject: false,
  stdin: "ignore",
};

/**
 * Merges default options with user-provided options and sets stdin to 'pipe' if input is provided.
 */
function mergedOptions(userOptions?: Options): Options {
  // we do some LD_LIBRARY_PATH tomfoolery to ensure a deno compile'd bin works
  // in alpine, but this breaks sub-processes, so we need to unset it here.
  // See flake.nix and rootfs_build.sh for more details.
  return {
    ...defaultOptions,
    ...userOptions,
    stdin: userOptions?.input ? "pipe" : "ignore",
    env: { LD_LIBRARY_PATH: "" },
  };
}

/**
 * Calculates exponential backoff delay with optional jitter
 */
function calculateDelay(
  attempt: number,
  baseDelay: number,
  maxDelay: number,
  useJitter: boolean = true
): number {
  const exponentialDelay = Math.min(baseDelay * Math.pow(2, attempt - 1), maxDelay);
  if (useJitter) {
    const jitter = Math.random() * 0.3 * exponentialDelay;
    return exponentialDelay + jitter;
  }
  return exponentialDelay;
}

/**
 * Promise-based delay utility
 */
function delay(ms: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeoutCb(() => resolve(), ms);
  });
}

export const makeExec = (_executionId: string) => {
  /**
   * Runs a command and waits until it finishes executing.
   *
   * @example
   * const child = siExec.waitUntilEnd("aws", [
   *   "ec2",
   *   "describe-hosts"
   * ]);
   */
  async function waitUntilEnd(
    execaFile: string,
    execaArgs?: readonly string[],
    execaOptions?: Options,
  ): Promise<SiExecResult> {
    console.log(
      `Running CLI command: "${execaFile} ${
        execaArgs
          ?.map((a) => `'${a}'`)
          ?.join(" ")
      }"`,
    );

    const child = await execa(
      execaFile,
      execaArgs,
      mergedOptions(execaOptions),
    );
    return child;
  }

  async function watch(
    options: WatchArgs,
    deadlineCount?: number,
  ): Promise<WatchResult> {
    if (!options.retryMs) {
      options.retryMs = 2000;
    }
    if (!options.maxRetryCount) {
      options.maxRetryCount = 10;
    }
    if (!deadlineCount) {
      deadlineCount = 0;
    }
    const c = await waitUntilEnd(
      options.cmd,
      options.args,
      options.execaOptions,
    );
    // Update the count of how many attempts we have made
    deadlineCount += 1;

    // If the process fails, fail immediately
    if (c.failed) {
      return { result: c, failed: "commandFailed" };
    }

    // If the deadline exceeded, fail
    if (deadlineCount >= options.maxRetryCount) {
      return { result: c, failed: "deadlineExceeded" };
    }

    // Evaluate the callback, and return if it found what it was looking for
    const o = await options.callback(c);
    if (o) {
      return { result: c };
    } else {
      return await setTimeout(options.retryMs, watch(options, deadlineCount));
    }
  }

  /**
   * Executes a function with exponential backoff retry logic
   */
  async function withRetry<T>(
    fn: () => Promise<T>,
    options: RetryOptions = {}
  ): Promise<RetryResult<T>> {
    const {
      maxAttempts = 20,
      baseDelay = 1000,
      maxDelay = 90000,
      jitter = true,
      isRateLimitedFn
    } = options;

    let lastError: any;

    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      try {
        const result = await fn();
        console.log(`[RETRY] Operation successful on attempt ${attempt}`);
        return { result, attempts: attempt };
      } catch (error) {
        lastError = error;
        
        // Check if this is a rate limiting error that should be retried
        const isRateLimited = isRateLimitedFn ? isRateLimitedFn(error) : false;
        
        if (attempt < maxAttempts && isRateLimited) {
          const delayMs = calculateDelay(attempt, baseDelay, maxDelay, jitter);
          console.log(`[RETRY] Rate limited on attempt ${attempt}, waiting ${Math.round(delayMs)}ms before retry`);
          await delay(delayMs);
          continue;
        } else if (attempt < maxAttempts && !isRateLimitedFn) {
          // If no rate limit function provided, retry all errors
          const delayMs = calculateDelay(attempt, baseDelay, maxDelay, jitter);
          console.log(`[RETRY] Error on attempt ${attempt}, waiting ${Math.round(delayMs)}ms before retry`);
          await delay(delayMs);
          continue;
        } else {
          // Max attempts reached or non-retryable error
          console.error(`[RETRY] Failed after ${attempt} attempts:`, error);
          throw lastError;
        }
      }
    }

    throw lastError;
  }

  /**
   * Polls a long-running operation until completion
   */
  async function pollLRO(options: LROPollOptions): Promise<any> {
    const {
      url,
      headers,
      maxAttempts = 20,
      baseDelay = 2000,
      maxDelay = 30000,
      isCompleteFn,
      isErrorFn,
      extractResultFn
    } = options;

    console.log(`[LRO] Starting polling for: ${url}`);

    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      console.log(`[LRO] Poll attempt ${attempt}`);

      const response = await fetch(url, {
        method: "GET",
        headers
      });

      let body: any;
      try {
        body = await response.json();
      } catch {
        // If JSON parsing fails, use text
        body = await response.text();
      }

      // Check if operation failed
      if (isErrorFn && isErrorFn(response, body)) {
        console.error(`[LRO] Operation failed:`, body);
        throw new Error(`LRO operation failed: ${JSON.stringify(body)}`);
      }

      // Check if operation completed
      if (isCompleteFn(response, body)) {
        console.log(`[LRO] Operation completed on attempt ${attempt}`);
        return extractResultFn ? await extractResultFn(response, body) : body;
      }

      // Continue polling
      if (attempt < maxAttempts) {
        const delayMs = calculateDelay(attempt, baseDelay, maxDelay, true);
        console.log(`[LRO] Waiting ${Math.round(delayMs)}ms before next poll`);
        await delay(delayMs);
      }
    }

    throw new Error(`LRO polling timeout after ${maxAttempts} attempts`);
  }

  /**
   * Enhanced command execution with built-in retry logic
   */
  async function waitUntilEndWithRetry(
    execaFile: string,
    execaArgs?: readonly string[],
    execaOptions?: Options,
    retryOptions?: RetryOptions
  ): Promise<RetryResult<SiExecResult>> {
    return withRetry(
      () => waitUntilEnd(execaFile, execaArgs, execaOptions),
      retryOptions
    );
  }

  return { waitUntilEnd, watch, withRetry, pollLRO, waitUntilEndWithRetry };
};

// export async function siExecStream(
//  ws: WebSocket,
//  execaFile: string,
//  execaArgs?: readonly string[],
//  execaOptions?: execa.Options<string>,
// ): Promise<SiExecResult> {
//  console.log(`running command; cmd="${execaFile} ${execaArgs?.join(" ")}"`);
//  ws.send(
//    JSON.stringify({
//      protocol: {
//        output: {
//          outputLine: `running command; cmd="${execaFile} ${execaArgs?.join(
//            " ",
//          )}"\n`,
//        },
//      },
//    }),
//  );
//
//  let stdout = "";
//  let stderr = "";
//  let all = "";
//
//  const child = execa(execaFile, execaArgs, {
//    stdout: "pipe",
//    stderr: "pipe",
//    all: true,
//    buffer: false,
//    ...execaOptions,
//  });
//
//  if (child.stdout) {
//    const stdoutRl = readline.createInterface({
//      input: child.stdout,
//      crlfDelay: Infinity,
//    });
//    stdoutRl.on("line", (data) => {
//      ws.send(
//        JSON.stringify({
//          protocol: {
//            output: {
//              outputLine: `${data}\n`,
//            },
//          },
//        }),
//      );
//      stdout = stdout + data;
//    });
//  }
//  if (child.stderr) {
//    const stderrRl = readline.createInterface({
//      input: child.stderr,
//      crlfDelay: Infinity,
//    });
//    stderrRl.on("line", (data) => {
//      ws.send(
//        JSON.stringify({
//          protocol: {
//            output: {
//              errorLine: `${data}\n`,
//            },
//          },
//        }),
//      );
//      stderr = stderr + data;
//    });
//  }
//  if (child.all) {
//    const allRl = readline.createInterface({
//      input: child.all,
//      crlfDelay: Infinity,
//    });
//    allRl.on("line", (data) => {
//      all = all + data;
//    });
//  }
//  const r = await child;
//  r.stdout = stdout;
//  r.stderr = stderr;
//  r.all = all;
//  return r;
// }
