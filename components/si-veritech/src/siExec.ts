import execa from "execa";
import readline from "readline";
import Debug from "debug";
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
