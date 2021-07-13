import api from "@opentelemetry/api";
import { HttpTraceContextPropagator } from "@opentelemetry/core";
import { CollectorTraceExporter } from "@opentelemetry/exporter-collector-grpc";
import { registerInstrumentations } from "@opentelemetry/instrumentation";
import { HttpInstrumentation } from "@opentelemetry/instrumentation-http";
import { KoaInstrumentation } from "@opentelemetry/instrumentation-koa";
import { NodeTracerProvider } from "@opentelemetry/node";
import { Resource } from "@opentelemetry/resources";
import { ResourceAttributes } from "@opentelemetry/semantic-conventions";
import {
  ConsoleSpanExporter,
  SimpleSpanProcessor,
} from "@opentelemetry/tracing";

const exporter = new CollectorTraceExporter({});
const provider = new NodeTracerProvider({
  resource: new Resource({
    [ResourceAttributes.SERVICE_NAME]: "si-veritech",
  }),
});

provider.addSpanProcessor(new SimpleSpanProcessor(exporter));
provider.addSpanProcessor(new SimpleSpanProcessor(new ConsoleSpanExporter()));
provider.register();

registerInstrumentations({
  instrumentations: [new KoaInstrumentation(), new HttpInstrumentation()],
  tracerProvider: provider,
});

api.propagation.setGlobalPropagator(new HttpTraceContextPropagator());

require("http");

export const tracer = api.trace.getTracer("si-veritech");
