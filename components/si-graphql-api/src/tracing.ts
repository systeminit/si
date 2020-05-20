import { NodeTracerProvider } from "@opentelemetry/node";
const provider = new NodeTracerProvider({
  plugins: {
    grpc: { enabled: true, path: "@opentelemetry/plugin-grpc" },
    http: { enabled: true, path: "@opentelemetry/plugin-http" },
    https: { enabled: true, path: "@opentelemetry/plugin-https" },
  },
});
import { SimpleSpanProcessor } from "@opentelemetry/tracing";
import { JaegerExporter } from "@opentelemetry/exporter-jaeger";
const collectorOptions = {
  serviceName: "si-graphql-api",
};
const exporter = new JaegerExporter(collectorOptions);
provider.addSpanProcessor(new SimpleSpanProcessor(exporter));
provider.register();
