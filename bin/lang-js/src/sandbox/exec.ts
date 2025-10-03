import { setTimeout } from "node:timers/promises";
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

  return { waitUntilEnd, watch };
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
