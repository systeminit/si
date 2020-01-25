import dotenv from "dotenv";

// Load the environment
dotenv.config();

const defaultPort = 4000;

interface Environment {
  apollo: {
    introspection: boolean;
    playground: boolean;
  };
  port: number | string;
  nodeEnv: string;
  logLevel: string;
  jwtKey: string;
  services: {
    "si-ssh-key": string;
    "si-account": string;
  };
}

export const environment: Environment = {
  apollo: {
    introspection: process.env.APOLLO_INTROSPECTION === "true",
    playground: process.env.APOLLO_PLAYGROUND === "true",
  },
  port: process.env.PORT || defaultPort,
  nodeEnv: process.env.NODE_ENV || "development",
  logLevel: process.env.LOG_LEVEL || "info",
  jwtKey: process.env.JWT_KEY || "slithering0d00risLithgeringpoler",
  services: {
    "si-account": process.env.SERVICES_SI_ACCOUNT || "127.0.0.1:5151",
    "si-ssh-key": process.env.SERVICES_SI_SSH_KEY || "127.0.0.1:5152",
  },
};
