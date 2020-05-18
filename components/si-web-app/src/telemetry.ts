import {
  ConsoleSpanExporter,
  SimpleSpanProcessor,
} from "@opentelemetry/tracing";
import { WebTracerProvider } from "@opentelemetry/web";
import { DocumentLoad } from "@opentelemetry/plugin-document-load";
import { ZoneContextManager } from "@opentelemetry/context-zone";
//import { CollectorExporter } from "@opentelemetry/exporter-collector";

const provider = new WebTracerProvider({
  plugins: [new DocumentLoad() as any],
});
provider.addSpanProcessor(new SimpleSpanProcessor(new ConsoleSpanExporter()));
//provider.addSpanProcessor(new SimpleSpanProcessor(new CollectorExporter()));

provider.register({
  contextManager: new ZoneContextManager(),
});

export const tracer = provider.getTracer("si-web-app");
let span = tracer.startSpan("poop");
span.end();
