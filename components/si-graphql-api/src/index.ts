import { ApolloServer } from "apollo-server";
import * as path from "path";
import * as jwtLib from "jsonwebtoken";
import { makeSchema } from "nexus";

import { services } from "@/services";
import { environment } from "@/environment";
//import { protobufLoader } from "@/protobuf";
//import { GraphqlHintLoader } from "@/graphql-hint";
//import { SchemaGenerator } from "@/schema/generator";
import { SiRegistryGenerator } from "@/schema/registryGenerator";
import { GrpcServiceBroker, Grpc } from "@/datasources/grpc";
import { DataSources } from "apollo-server-core/dist/graphqlOptions";

import "@/schema/registryGenerator";

// First, load the protocol buffers
//const protobufLoader = new ProtobufLoader({
//  protos: [
//    //path.join(__dirname, "..", "..", "si-data", "proto", "si.data.proto"),
//  ],
//  services,
//});

// Second, load the graphql hints
//const graphqlHintLoader = new GraphqlHintLoader({
//  services,
//});

// Pass them to our custom schema generator
//const sg = new SchemaGenerator(services, protobufLoader, graphqlHintLoader);
//sg.generate();
//

const sg = new SiRegistryGenerator();
sg.generate();

const schema = makeSchema({
  types: sg.types,
  nonNullDefaults: { output: false, input: false },
  outputs: {
    schema: path.join(__dirname, "../fullstack-schema.graphql"),
    typegen: path.join(
      __dirname.replace(/\/dist$/, "/src"),
      "../src/fullstack-typegen.ts",
    ),
  },
  typegenAutoConfig: {
    sources: [
      {
        source: path.join(
          __dirname.replace(/\/dist$/, "/src"),
          "./typeDefs.ts",
        ),
        alias: "t",
      },
    ],
    debug: true,
    //contextType: "t.Context",
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
    console.log("-------------------ERROR----------------------");
    console.dir(error, { depth: Infinity });
    return error;
  },
  formatResponse: response => {
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
      console.log("-------------------REQUEST----------------------");
      console.dir(req.body, { depth: Infinity });
      const token = req.headers.authorization || "";
      const userContext: UserContext = { authenticated: false };
      if (token.startsWith("Bearer ")) {
        const authParts = token.split(" ");
        const payload = jwtLib.verify(authParts[1], environment.jwtKey, {
          audience: "https://app.systeminit.com",
          issuer: "https://app.systeminit.com",
          clockTolerance: 60,
        });
        if (payload["billingAccountId"] && payload["userId"]) {
          userContext["authenticated"] = true;
          userContext["billingAccountId"] = payload["billingAccountId"];
          userContext["userId"] = payload["userId"];
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
