import dotenv from "dotenv";

// Load the environment
dotenv.config();

const defaultPort = 4001;

interface Environment {
  port: number | string;
  nodeEnv: string;
  logLevel: string;
  jwtKey: string;
}

export const environment: Environment = {
  port: process.env.PORT || defaultPort,
  nodeEnv: process.env.NODE_ENV || "development",
  logLevel: process.env.LOG_LEVEL || "info",
  jwtKey: process.env.JWT_KEY || "slithering0d00risLithgeringpoler",
};
