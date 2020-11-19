import execa from "execa";
import readline from "readline";
import { Event, EventLogLevel } from "./eventLog";

export async function siExec(
  event: Event,
  execaFile: string,
  execaArgs?: readonly string[],
  execaOptions?: execa.Options<string>,
): Promise<execa.ExecaReturnValue<string>> {
  const eventLog = event.log(
    EventLogLevel.Info,
    `running ${execaFile} ${execaArgs?.join(" ")}`,
    {
      command: execaFile,
      args: execaArgs,
    },
  );

  let stdout = "";
  let stderr = "";
  let all = "";

  try {
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
      stdoutRl.on("line", data => {
        eventLog.output("stdout", data);
        stdout = stdout + data;
      });
    }
    if (child.stderr) {
      const stderrRl = readline.createInterface({
        input: child.stderr,
        crlfDelay: Infinity,
      });
      stderrRl.on("line", data => {
        eventLog.output("stderr", data);
        stderr = stderr + data;
      });
    }
    if (child.all) {
      const allRl = readline.createInterface({
        input: child.all,
        crlfDelay: Infinity,
      });
      allRl.on("line", data => {
        eventLog.output("all", data);
        all = all + data;
      });
    }
    const r = await child;
    r.stdout = stdout;
    r.stderr = stderr;
    r.all = all;
    return r;
  } catch (err) {
    if (err.stdout) {
      eventLog.output("stdout", err.stdout);
    }
    if (err.stderr) {
      eventLog.output("stderr", err.stderr);
    }
    if (err.all) {
      eventLog.output("stderr", err.all);
    }
    eventLog.message = `${eventLog.message} failed`;
    eventLog.payload["fatal"] = `${err}`;
    eventLog.fatal();
    throw err;
  }
}
