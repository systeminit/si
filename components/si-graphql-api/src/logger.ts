import * as winston from "winston";
import { environment } from "@/environment";

export const logger = winston.createLogger({
  level: environment.logLevel,
  defaultMeta: { service: "si-graphql-api" },
});

if (environment.nodeEnv !== "production") {
  logger.add(
    new winston.transports.Console({
      format: winston.format.combine(
        winston.format.timestamp(),
        winston.format.colorize(),
        winston.format.simple(),
        //winston.format.prettyPrint(),
      ),
    }),
  );
}
