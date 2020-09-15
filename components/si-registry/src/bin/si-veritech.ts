import program from "commander";
import { start } from "@/veritech/server";

program
  .version("0.0.1")
  .description("Entity Intelligence as a tiny robotech fighter")
  .option("-p, --port <port>", "port number", "5157")
  .parse(process.argv);

main(program);

function main(program: program.Command): void {
  start(program.port);
}
