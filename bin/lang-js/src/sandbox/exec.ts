import execa from "execa";
import { ExecaReturnValue, Options } from "execa";
import Debug from "debug";
const debug = Debug("langJs:siExec");

//import readline from "readline";
//import WebSocket from "ws";

export type SiExecResult = ExecaReturnValue<string>;

// Note(paulo): This is highly dangerous as it bypasses the sandbox
// We also are bypassing the VM timeout by using async (NodeVM doesn't have timeout, but it seems we can't await without it)
export const makeExec = (executionId: string) => {
  async function waitUntilEnd(
    execaFile: string,
    execaArgs?: readonly string[],
    execaOptions?: Options<string>
  ): Promise<SiExecResult> {
    debug(
      `running command; executionId="${executionId}"; cmd="${execaFile} ${execaArgs
        ?.map((a) => `'${a}'`)
        ?.join(" ")}"`
    );
    console.log(JSON.stringify({
      protocol: "output",
      executionId,
      stream: "stderr",
      level: "debug",
      group: "log",
      message: `Running CLI command: "${execaFile} ${execaArgs
        ?.map((a) => `'${a}'`)
        ?.join(" ")}"`
    }));

    const child = await execa(execaFile, execaArgs, {
      all: true,
      buffer: true,
      reject: false,
      ...execaOptions,
    });
    return child;
  }

  return { waitUntilEnd };
};

//export async function siExecStream(
//  ws: WebSocket,
//  execaFile: string,
//  execaArgs?: readonly string[],
//  execaOptions?: execa.Options<string>,
//): Promise<SiExecResult> {
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
//}
