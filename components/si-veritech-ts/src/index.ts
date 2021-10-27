import { app } from "./app";

export function start(port: number): void {
  const server = app.listen(port, () => {
    console.log(`Starting veritech on ${port}`);
  });
  // This is probably way, way too high. But still!
  server.keepAliveTimeout = 600000;
  server.headersTimeout = 601000;
}

start(5157);
