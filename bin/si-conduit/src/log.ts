export class Log {
  private readonly verbosity: number;

  constructor(verbosity: number = 0) {
    this.verbosity = verbosity;
  }

  private log(level: "ERROR" | "INFO" | "DEBUG", message: string): void {
    const timestamp = new Date().toISOString();
    console.log(`[${timestamp}] ${level}: ${message}`);
  }

  error(message: string): void {
    this.log("ERROR", message);
  }

  info(message: string): void {
    if (this.verbosity >= 1) {
      this.log("INFO", message);
    }
  }

  debug(message: string): void {
    if (this.verbosity >= 2) {
      this.log("DEBUG", message);
    }
  }
}
