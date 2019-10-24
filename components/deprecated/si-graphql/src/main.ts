import http from "http";
import app from "@/server";
import { environment } from "@/environment";

const server = http.createServer(app);

server.listen({ port: environment.port }, (): void => {
  console.log(`System Initiative GraphQL Listening on ${environment.port}`);
});
