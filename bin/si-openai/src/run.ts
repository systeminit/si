import { Command } from "https://deno.land/x/cliffy@v1.0.0-rc.3/command/mod.ts";
import { generateAwsSchema } from "./aws.ts";

export async function run() {
  const command = new Command()
    .name("si-openai")
    .version("0.1.0")
    .description("AI generated assets")
    .action(() => {
      command.showHelp();
      Deno.exit(1);
    })
    .command("asset")
    .description("generate an asset definition from an aws cli help document")
    .arguments("<awsService:string> <awsCommand:string>")
    .option(
      "--propertyFromUrl <input:string>",
      "property:url; instructs to get schema from url",
      { collect: true, required: false },
    )
    .action(async (options, awsService, awsCommand) => {
      const propsFromUrl = [];
      if (options.propertyFromUrl) {
        for (const propFromUrl of options.propertyFromUrl) {
          const parts = propFromUrl.split(":");
          propsFromUrl.push({ property: parts[0], url: parts.slice(1).join(":") });
        }
      }
      const result = await generateAwsSchema(
        awsService,
        awsCommand,
        propsFromUrl,
      );
      console.log(result);
    })
    .command("function")
    .description("generate an asset function definition from an aws cli help document")
    .arguments("<awsService:string> <awsCommand:string>")
    .option(
      "--propertyFromUrl <input:string>",
      "property:url; instructs to get schema from url",
      { collect: true, required: false },
    )
    .action(async (options, awsService, awsCommand) => {
      const propsFromUrl = [];
      if (options.propertyFromUrl) {
        for (const propFromUrl of options.propertyFromUrl) {
          const parts = propFromUrl.split(":");
          propsFromUrl.push({ property: parts[0], url: parts.slice(1).join(":") });
        }
      }
      const result = await generateAwsSchema(
        awsService,
        awsCommand,
        propsFromUrl,
      );
      console.log(result);
    });

  const _result = await command.parse(Deno.args);
}
