import { Command } from "@cliffy/command";
import { fetchSchema } from "./commands/fetchSchema.ts";
import { generateSiSpecDatabase } from "./commands/generateSiSpecDatabase.ts";

export async function run() {
  const command = new Command()
    .name("si-gen-cloud-control")
    .version("0.1.0")
    .description("Asset Pipeline for AWS Cloud Control")
    .env("LOG_LEVEL=<value:string>", "Set the log level; defaults to info")
    .action(() => {
      command.showHelp();
      Deno.exit(1);
    })
    .command(
      "fetch-schema",
      "fetch cloudformation schema from aws",
    )
    .action(async () => {
      await fetchSchema();
    })
    .command(
      "generate-spec-database",
      "generate the si spec database from the cf database",
    )
    .action(async () => {
      await generateSiSpecDatabase();
    });

  await command.parse(Deno.args);
}
