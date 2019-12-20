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
  couchbase: {
    cluster: string;
    bucket: string;
    username: string;
    password: string;
  };
}

export const environment: Environment = {
  apollo: {
    introspection: process.env.APOLLO_INTROSPECTION === "true",
    playground: process.env.APOLLO_PLAYGROUND === "true",
  },
  port: process.env.PORT || defaultPort,
  nodeEnv: process.env.NODE_ENV || "development",
  couchbase: {
    cluster: process.env.COUCHBASE_CLUSTER || "couchbase://127.0.0.1",
    bucket: process.env.COUCHBASE_BUCKET || "si",
    username: process.env.COUCHBASE_USERNAME || "si",
    password: process.env.COUCHBASE_PASSWORD || "bugbear",
  },
};
