import { Command } from "@cliffy/command";
import { fetchSchema } from "./commands/fetchSchema.ts";
import { generateSiSpecs } from "./commands/generateSiSpecs.ts";
import { generateTarFromSpec } from "./commands/generateTarFromSpec.ts";

const DEFAULT_MODULE_INDEX_URL = "http://0.0.0.0:5157";

export async function run() {
  const command = new Command()
    .name("clover")
    .version("0.1.0")
    .description("Asset Pipeline for AWS Cloud Control")
    .globalEnv(
      "LOG_LEVEL=<value:string>",
      "Set the log level; defaults to info"
    )
    .action(() => {
      command.showHelp();
      Deno.exit(1);
    })
    .command(
      "fetch-schema",
      "Getch cloudformation schema from aws.",
    )
    .action(async () => {
      await fetchSchema();
    })
    .command(
      "generate-specs [...services:string]",
      `Generate the si spec database from the cf database.

To generate all specs:

  clover generate-specs

To generate all specs containing "ECS" or "S3", you can pass some services as arguments:

  clover generate-specs ECS S3
`
    )
    .env(
      "SI_BEARER_TOKEN=<value:string>",
      "Auth token for interacting with the module index",
      { required: true },
    )
    .env(
      "SI_MODULE_INDEX_URL=<value:string>",
      "Set the module index url; defaults to http://0.0.0.0:5157",
      { prefix: "SI_" }
    )
    .action(async ({ moduleIndexUrl }, ...services: string[]) => {
      await generateSiSpecs(moduleIndexUrl ?? DEFAULT_MODULE_INDEX_URL, services);
    })
    .command(
      "generate-tars",
      "Generate tar packages from the spec files in si-specs",
    )
    .action(async () => {
      await generateTarFromSpec();
    });

  await command.parse(Deno.args);
}
