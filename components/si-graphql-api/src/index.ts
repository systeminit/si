import { NodeTracerProvider } from "@opentelemetry/node";
import { SimpleSpanProcessor } from "@opentelemetry/tracing";
import { JaegerExporter } from "@opentelemetry/exporter-jaeger";
import api, { CanonicalCode } from "@opentelemetry/api";

const collectorOptions = {
  serviceName: "si-graphql-api",
};
const provider = new NodeTracerProvider();
const exporter = new JaegerExporter(collectorOptions);
provider.addSpanProcessor(new SimpleSpanProcessor(exporter));
provider.register();
const tracer = api.trace.getTracer("si-graphql-api");

import { ApolloServer } from "apollo-server";
import * as path from "path";
import * as jwtLib from "jsonwebtoken";
import { makeSchema } from "nexus";

import { services } from "@/services";
import { environment } from "@/environment";
import { registryGenerator } from "@/schema/registryGenerator";
import { loginTypes } from "@/schema/login";
import { GrpcServiceBroker, Grpc } from "@/datasources/grpc";
import { DataSources } from "apollo-server-core/dist/graphqlOptions";

import "@/schema/registryGenerator";

registryGenerator.generate();
const graphqlTypes = [registryGenerator.types, loginTypes];

const schema = makeSchema({
  types: graphqlTypes,
  nonNullDefaults: { output: false, input: false },
  outputs: {
    schema: path.resolve("./fullstack-schema.graphql"),
    typegen: path.resolve("./fullstack-typegen.ts"),
  },
});

const serviceBroker = new GrpcServiceBroker({ services });

export interface DataSourceContext {
  grpc: any;
}

export interface UserContext {
  authenticated: boolean;
  userId?: string;
  billingAccountId?: string;
}

export interface Context {
  dataSources?: DataSourceContext;
  user: UserContext;
  associationParent?: any;
}

const dataSources = (): DataSources<DataSourceContext> => ({
  grpc: new Grpc({ broker: serviceBroker }),
});

const server = new ApolloServer({
  cors: {
    origin: "*", // <- allow request from all domains; public api
    credentials: true,
  },
  schema,
  dataSources,
  formatError: error => {
    let span = tracer.getCurrentSpan();
    if (span == undefined) {
      span = tracer.startSpan("request without root span");
    }
    if (error && error.message) {
      span.setAttributes({
        error: true,
        //"error.path": error.path,
        //"error.name": error.name,
        //"error.source.name": error.source.name,
        //"error.source.body": error.source.body,
        "error.message": error.message,
        //"error.originalError": `${error.originalError}`,
      });
    } else {
      span.setAttributes({ error: true });
    }
    span.setStatus({ code: CanonicalCode.UNKNOWN });
    console.log("-------------------ERROR----------------------");
    console.dir(error, { depth: Infinity });
    return error;
  },
  formatResponse: (response: any) => {
    let span = tracer.getCurrentSpan();
    if (span == undefined) {
      span = tracer.startSpan("request without root span");
    }
    span.addEvent("graphql response", { response });
    console.log("-------------------RESPONSE----------------------");
    console.dir(response, { depth: Infinity });
    return response;
  },
  context: ({ req, connection }): Context => {
    if (connection) {
      return {
        //@ts-ignore
        dataSources,
        connection,
      };
      //console.log({ connection });
    } else {
      let span = tracer.getCurrentSpan();
      if (span == undefined) {
        span = tracer.startSpan("request without root span");
      }
      span.addEvent("graphql request", {
        "graphql.request.query": req.body.query,
        "graphql.request.variables": JSON.stringify(req.body.variables),
      });
      console.log("-------------------REQUEST----------------------");
      console.dir(req.body, { depth: Infinity });
      const token = req.headers.authorization || "";
      const userContext: UserContext = { authenticated: false };
      if (token.startsWith("Bearer ")) {
        const authParts = token.split(" ");
        const payload: string | Record<string, any> = jwtLib.verify(
          authParts[1],
          environment.jwtKey,
          {
            audience: "https://app.systeminit.com",
            issuer: "https://app.systeminit.com",
            clockTolerance: 60,
          },
        );
        if (
          typeof payload != "string" &&
          payload["billingAccountId"] &&
          payload["userId"]
        ) {
          userContext["authenticated"] = true;
          span.setAttribute("authenticated", true);
          userContext["billingAccountId"] = payload["billingAccountId"];
          span.setAttribute("billingAccountId", payload["billingAccountId"]);
          userContext["userId"] = payload["userId"];
          span.setAttribute("userId", payload["userId"]);
        }
      }
      return { user: userContext };
    }
  },
});

const port = process.env.PORT || 4000;

server.listen({ port }, () =>
  console.log(
    `==> Server ready at http://0.0.0.0:${port}${server.graphqlPath} <==`,
  ),
);
