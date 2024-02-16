import { Command } from "https://deno.land/x/cliffy@v1.0.0-rc.3/command/mod.ts";
import { awsGenerate } from "./asset_generator.ts";
import { renderAsset } from "./render.ts";

export async function run() {
  const command = new Command()
    .name("si-generator")
    .version("0.1.0")
    .description(
      "Generate Assets and code for System Initiative",
    )
    .action(() => {
      command.showHelp();
      Deno.exit(1);
    })
    .command("asset")
    .description("generate an asset definition from an aws cli skeleton")
    .arguments("<awsService:string> <awsCommand:string>")
    .action(async (_options, awsService, awsCommand) => {
      const props = awsGenerate(awsService, awsCommand);
      const result = await renderAsset(props, "aws");
      console.log(result);
    });
  const _result = await command.parse(Deno.args);
}
