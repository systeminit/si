import { OutputLine } from "../function";

const normalizeMessage = (message: unknown): string => {
  if (typeof(message) === typeof("")) return message as string;
  return JSON.stringify(message ?? "");
}

export const makeConsole = (executionId: string) => {
  function debug(message: unknown, data: unknown): void {
    emitOutputLine({
      protocol: "output",
      executionId,
      stream: "stdout",
      level: "debug",
      group: "log",
      message: normalizeMessage(message),
      data,
    });
  }

  function error(message: unknown, data: unknown): void {
    emitOutputLine({
      protocol: "output",
      executionId,
      stream: "stderr",
      level: "error",
      group: "log",
      message: normalizeMessage(message),
      data,
    });
  }

  function log(message: unknown, data: unknown): void {
    emitOutputLine({
      protocol: "output",
      executionId,
      stream: "stdout",
      level: "info",
      group: "log",
      message: normalizeMessage(message),
      data,
    });
  }

  function emitOutputLine(line: OutputLine): void {
    console.log(JSON.stringify(line));
  }

  return { debug, error, log };
};
