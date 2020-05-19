import {
  ConsoleSpanExporter,
  SimpleSpanProcessor,
} from "@opentelemetry/tracing";
import { WebTracerProvider } from "@opentelemetry/web";
import { DocumentLoad } from "@opentelemetry/plugin-document-load";
import { UserInteractionPlugin } from "@opentelemetry/plugin-user-interaction";
import { ZoneContextManager } from "@opentelemetry/context-zone";
import { CollectorExporter } from "@opentelemetry/exporter-collector";

const provider = new WebTracerProvider({
  plugins: [new DocumentLoad() as any, new UserInteractionPlugin()],
});
provider.addSpanProcessor(
  new SimpleSpanProcessor(
    new CollectorExporter({
      serviceName: "si-web-app",
    }),
  ),
);
provider.addSpanProcessor(new SimpleSpanProcessor(new ConsoleSpanExporter()));
provider.register({
  contextManager: new ZoneContextManager(),
});

export const tracer = provider.getTracer("si-web-app");
