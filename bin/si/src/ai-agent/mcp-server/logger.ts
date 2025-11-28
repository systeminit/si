import { ConsoleStream, Level, Logger, type LogRecord } from "@onjara/optic";
import { TokenReplacer } from "@onjara/optic/formatters";
import type _ from "lodash";

class StderrStream extends ConsoleStream {
  override log(msg: string): void {
    console.error(msg);
  }

  override handle(logRecord: LogRecord): boolean {
    if (this.minLevel > logRecord.level) return false;
    const msg = this.format(logRecord);

    if (logRecord.level >= Level.Error) {
      console.error(msg);
    } else if (logRecord.level >= Level.Warn) {
      console.error(msg);
    } else if (logRecord.level >= Level.Info) {
      console.error(msg);
    } else if (logRecord.level >= Level.Debug) {
      console.error(msg);
    } else {
      console.error(msg);
    }

    return true;
  }
}

export const logger = new Logger()
  .addStream(new StderrStream().withFormat(
    new TokenReplacer(),
  ))
  .withMinLogLevel(Level.Debug);

export function debugLogFile(data: unknown) {
  Deno.writeTextFileSync(
    "/tmp/mcp-debug.txt",
    `${JSON.stringify(data, null, 2)}\n`,
    { append: true },
  );
}
