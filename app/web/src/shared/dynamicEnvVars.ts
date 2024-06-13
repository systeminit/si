type ProjectEnvVariablesType = Pick<
  ImportMetaEnv,
  "VITE_OTEL_EXPORTER_OTLP_ENDPOINT"
>;

// We must use `${}` so this variable gets replaced in the docker container
const projectEnvVariables: ProjectEnvVariablesType = {
  VITE_OTEL_EXPORTER_OTLP_ENDPOINT: "${VITE_OTEL_EXPORTER_OTLP_ENDPOINT}", // eslint-disable-line no-template-curly-in-string
};

// Returning the variable value from runtime or obtained as a result of the build
export const getProjectEnvVariables = (): {
  envVariables: ProjectEnvVariablesType;
} => {
  return {
    envVariables: {
      VITE_OTEL_EXPORTER_OTLP_ENDPOINT:
        !projectEnvVariables.VITE_OTEL_EXPORTER_OTLP_ENDPOINT.includes("VITE_")
          ? projectEnvVariables.VITE_OTEL_EXPORTER_OTLP_ENDPOINT
          : import.meta.env.VITE_OTEL_EXPORTER_OTLP_ENDPOINT,
    },
  };
};
