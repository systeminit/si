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
  mqttBrokerUrl: string;
  services: {
    "si-ssh-key": string;
    "si-account": string;
    "si-aws-eks-cluster-runtime": string;
    "si-kubernetes": string;
  };
}

export const environment: Environment = {
  apollo: {
    introspection: process.env.APOLLO_INTROSPECTION === "true",
    playground: process.env.APOLLO_PLAYGROUND === "true",
  },
  mqttBrokerUrl: process.env.MQTT_BROKER_URL || "mqtt://localhost",
  port: process.env.PORT || defaultPort,
  nodeEnv: process.env.NODE_ENV || "development",
  logLevel: process.env.LOG_LEVEL || "info",
  jwtKey: process.env.JWT_KEY || "slithering0d00risLithgeringpoler",
  services: {
    "si-account": process.env.SERVICES_SI_ACCOUNT || "127.0.0.1:5151",
    "si-ssh-key": process.env.SERVICES_SI_SSH_KEY || "127.0.0.1:5152",
    "si-aws-eks-cluster-runtime":
      process.env.SERVICES_SI_AWS_EKS_CLUSTER_RUNTIME || "127.0.0.1:5154",
    "si-kubernetes": process.env.SERVICES_SI_KUBERNETES || "127.0.0.1:5155",
  },
};
