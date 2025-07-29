import { ConsoleStream, Level, Logger, LogRecord } from "@onjara/optic";
import { TokenReplacer } from "@onjara/optic/formatters";

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
