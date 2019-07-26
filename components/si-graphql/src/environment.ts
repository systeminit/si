import dotenv from 'dotenv';

// Load the environment
dotenv.config();

const defaultPort = 4000;
const defaultNodeEnv = 'development';

interface Environment {
  apollo: {
    introspection: boolean;
    playground: boolean;
  };
  port: number | string;
  node_env: string;
}

export const environment: Environment = {
  apollo: {
    introspection: process.env.APOLLO_INTROSPECTION === 'true',
    playground: process.env.APOLLO_PLAYGROUND === 'true',
  },
  port: process.env.PORT || defaultPort,
  node_env: process.env.NODE_ENV || 'development',
};
