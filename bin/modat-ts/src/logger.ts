import * as wf from "@temporalio/workflow";

export interface LoggerSinks extends wf.Sinks {
  logger: {
    error(message: string): void;
    info(message: string, data?: unknown): void;
  };
}

export const { logger } = wf.proxySinks<LoggerSinks>();
