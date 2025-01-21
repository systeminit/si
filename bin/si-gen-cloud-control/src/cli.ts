import { Command } from "@cliffy/command";
import { fetchSchema } from "./commands/fetchSchema.ts";
import { generateSiSpecs } from "./commands/generateSiSpecs.ts";
import { generateTarFromSpec } from "./commands/generateTarFromSpec.ts";

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
      "generate-specs",
      "generate the si spec database from the cf database",
    )
    .action(async () => {
      await generateSiSpecs();
    })
    .command(
      "generate-tars",
      "generate tar packages from the spec files in si-specs",
    )
    .action(async () => {
      await generateTarFromSpec();
    });

  await command.parse(Deno.args);
}
