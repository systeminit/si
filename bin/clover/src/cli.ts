import path from "node:path";
import { Command, EnumType } from "jsr:@cliffy/command@^1.0.0-rc.8";
import { fetchSchema } from "./commands/fetchSchema.ts";
import { generateSiSpecs } from "./commands/generateSiSpecs.ts";
import { generateTarFromSpec } from "./commands/generateTarFromSpec.ts";
import { inferAi } from "./commands/inferAi.ts";
import type { Provider } from "./types.ts";
import { PROVIDER_REGISTRY } from "./pipelines/types.ts";

const DEFAULT_MODULE_INDEX_URL = "http://0.0.0.0:5157";
const DEFAULT_PROVIDER_SCHEMAS_PATH = path.join(
  import.meta.dirname!,
  "provider-schemas",
);
export async function run() {
  const command = new Command()
    .name("clover")
    .version("0.1.0")
    .description("Asset Pipeline for AWS Cloud Control")
    .globalEnv(
      "LOG_LEVEL=<value:string>",
      "Set the log level; defaults to info",
    )
    .globalOption(
      "--provider-schemas-path <path:string>",
      "Path where provider-specific schemas are cached.",
      {
        default: DEFAULT_PROVIDER_SCHEMAS_PATH,
      },
    )
    .globalType(
      "provider",
      new EnumType<Provider>([...Object.keys(PROVIDER_REGISTRY), "all"]),
    )
    .globalOption(
      "-p, --provider=<value:provider>",
      "The specific provider to generate specs for",
      {
        required: true,
      },
    )
    .action(() => {
      command.showHelp();
      Deno.exit(1);
    })
    // fetch-schema
    .command("fetch-schema", "Getch cloudformation schema from aws.")
    .action(async (options) => await fetchSchema(options))
    // generate-specs
    .command(
      "generate-specs [...services:string]",
      `Generate the si spec database from the cf database.

To generate all specs:

  clover generate-specs

To generate all specs containing "ECS" or "S3", you can pass some services as arguments:

  clover generate-specs ECS S3
`,
    )
    .option("--inferred <file:string>", "Inferred database location", {
      default: "inferred.json",
    })
    .option("--doc-link-cache <file:string>", "Cache of doc link status", {
      default: "doc-link-cache.json",
    })
    .option(
      "--force-update-existing-packages",
      "Force the existing package list to be updated",
    )
    .env(
      "SI_BEARER_TOKEN=<value:string>",
      "Auth token for interacting with the module index",
      { required: true },
    )
    .env(
      "SI_MODULE_INDEX_URL=<value:string>",
      "Set the module index url; defaults to http://0.0.0.0:5157",
      { prefix: "SI_" },
    )
    .action(async (options, ...services: string[]) => {
      await generateSiSpecs({
        ...options,
        moduleIndexUrl: options.moduleIndexUrl ?? DEFAULT_MODULE_INDEX_URL,
        services: services?.length > 0 ? services : undefined,
      });
    })
    // infer-ai
    .command(
      "infer-ai [...services:string]",
      `Prompt the AI to give existing Cloud Formation specs based on their descriptions.

To run inference on all specs:

  clover infer-ai

To run inference on all specs whose type names contain "ECS" or "S3", you can pass some services as arguments:

  clover infer-ai ECS S3

By default, the AI will only run inference for things that have not already been inferred. To
force the AI to re-infer all specs, pass the --force flag.
`,
    )
    .option("--force", "Force re-inference on cached values")
    .option("--inferred <file:string>", "Inferred database location", {
      default: "inferred.json",
    })
    .env("OPENAI_API_KEY=<value:string>", "OpenAI AI token", { required: true })
    .action(async (options, ...services: string[]) => {
      await inferAi({
        ...options,
        services: services?.length > 0 ? services : undefined,
      });
    })
    // generate-tars
    .command(
      "generate-tars",
      "Generate tar packages from the spec files in si-specs",
    )
    .action(async () => {
      await generateTarFromSpec();
    });

  await command.parse(Deno.args);
}
