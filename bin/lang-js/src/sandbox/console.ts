import { OutputLine } from "../function.ts";
import { Debug } from "../debug.ts";

const normalizeMessage = (msg: unknown[]): string => {
  return msg
    .map((m) => {
      if (typeof m === typeof "") return m as string;
      return JSON.stringify(m);
    })
    .join(" ");
};

export const makeConsole = (executionId: string) => {
  function debug(...args: unknown[]): void {
    emitOutputLine({
      protocol: "output",
      executionId,
      stream: "stdout",
      level: "debug",
      group: "log",
      message: normalizeMessage(args),
    });
  }

  function error(...args: unknown[]): void {
    emitOutputLine({
      protocol: "output",
      executionId,
      stream: "stderr",
      level: "error",
      group: "log",
      message: normalizeMessage(args),
    });
  }

  function log(...args: unknown[]): void {
    emitOutputLine({
      protocol: "output",
      executionId,
      stream: "stdout",
      level: "info",
      group: "log",
      message: normalizeMessage(args),
    });
  }

  function emitOutputLine(line: OutputLine): void {
    console.log(JSON.stringify(line));
  }

  return { debug, error, log };
};
