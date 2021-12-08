import { OutputLine } from "../function";

export const makeConsole = (executionId: string) => {
  function debug(message: string, data: unknown): void {
    emitOutputLine({
      protocol: "output",
      executionId,
      stream: "stdout",
      level: "debug",
      group: "log",
      message,
      data,
    });
  }

  function error(message: string, data: unknown): void {
    emitOutputLine({
      protocol: "output",
      executionId,
      stream: "stderr",
      level: "error",
      group: "log",
      message,
      data,
    });
  }

  function log(message: string, data: unknown): void {
    emitOutputLine({
      protocol: "output",
      executionId,
      stream: "stdout",
      level: "info",
      group: "log",
      message,
      data,
    });
  }

  function emitOutputLine(line: OutputLine): void {
    console.log(JSON.stringify(line));
  }

  return { debug, error, log };
};
