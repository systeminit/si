import { RemoteFunctionOutputLine } from "../remote_function";

export function log(message: string, data: unknown): void {
  const outputLine: RemoteFunctionOutputLine = {
    protocol: "output",
    stream: "stdout",
    level: "info",
    group: "log",
    timestamp: new Date().getTime(),
    message,
    data,
  };
  console.log(JSON.stringify(outputLine));
}

export const consoleObject = { log };
