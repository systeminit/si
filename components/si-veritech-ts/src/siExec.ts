import execa from "execa";
import readline from "readline";
import Debug from "debug";
import WebSocket from "ws";
const debug = Debug("veritech:siExec");

export type SiExecResult = execa.ExecaReturnValue<string>;

export async function siExec(
  execaFile: string,
  execaArgs?: readonly string[],
  execaOptions?: execa.Options<string>,
): Promise<SiExecResult> {
  debug(`running command; cmd="${execaFile} ${execaArgs?.join(" ")}"`);

  const child = await execa(execaFile, execaArgs, {
    all: true,
    buffer: true,
    reject: false,
    ...execaOptions,
  });
  return child;
}

export async function siExecStream(
  ws: WebSocket,
  execaFile: string,
  execaArgs?: readonly string[],
  execaOptions?: execa.Options<string>,
): Promise<SiExecResult> {
  console.log(`running command; cmd="${execaFile} ${execaArgs?.join(" ")}"`);
  ws.send(
    JSON.stringify({
      protocol: {
        output: {
          outputLine: `running command; cmd="${execaFile} ${execaArgs?.join(
            " ",
          )}"\n`,
        },
      },
    }),
  );

  let stdout = "";
  let stderr = "";
  let all = "";

  const child = execa(execaFile, execaArgs, {
    stdout: "pipe",
    stderr: "pipe",
    all: true,
    buffer: false,
    ...execaOptions,
  });

  if (child.stdout) {
    const stdoutRl = readline.createInterface({
      input: child.stdout,
      crlfDelay: Infinity,
    });
    stdoutRl.on("line", (data) => {
      ws.send(
        JSON.stringify({
          protocol: {
            output: {
              outputLine: `${data}\n`,
            },
          },
        }),
      );
      stdout = stdout + data;
    });
  }
  if (child.stderr) {
    const stderrRl = readline.createInterface({
      input: child.stderr,
      crlfDelay: Infinity,
    });
    stderrRl.on("line", (data) => {
      ws.send(
        JSON.stringify({
          protocol: {
            output: {
              errorLine: `${data}\n`,
            },
          },
        }),
      );
      stderr = stderr + data;
    });
  }
  if (child.all) {
    const allRl = readline.createInterface({
      input: child.all,
      crlfDelay: Infinity,
    });
    allRl.on("line", (data) => {
      all = all + data;
    });
  }
  const r = await child;
  r.stdout = stdout;
  r.stderr = stderr;
  r.all = all;
  return r;
}
