import { Command } from "https://deno.land/x/cliffy@v1.0.0-rc.3/command/mod.ts";
import { awsGenerate } from "./asset_generator.ts";
import { makeRefreshOptions } from "./resource_generator.ts";
import { renderAction, renderAsset, renderCodeGen, renderCreate, renderDelete, renderRefresh } from "./render.ts";
import { makeDeleteOrActionOptions } from "./resource_generator.ts";

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
    })
    .command("codeGen")
    .description("generate a codeGen definition for an aws asset to create")
    .action(async (_options) => {
      const result = await renderCodeGen("aws");
      console.log(result);
    })
    .command("create")
    .description("generate a create function from an aws cli skeleton")
    .arguments("<awsService:string> <awsCommand:string>")
    .action(async (_options, awsService, awsCommand) => {
      const result = await renderCreate({ provider: "aws", awsService, awsCommand });
      console.log(result);
    })
    .command("refresh")
    .description("generate a refresh function from an aws cli skeleton")
    .arguments("<awsService:string> <awsCommand:string>")
    .option("--input <input:string>", "awsInputPath:siPropertiesPath; constructs CLI json data", { collect: true, required: true })
    .option("--output <output:string>", "[siResourcePath:]awsOutputPath; constructs resource object", { collect: true, required: true })
    .action(async (options, awsService, awsCommand) => {
      const refreshOptions = makeRefreshOptions(options);
      const result = await renderRefresh({ provider: "aws", awsService, awsCommand, inputs: refreshOptions.inputs, outputs: refreshOptions.outputs });
      console.log(result);
    })
    .command("delete")
    .description("generate a delete function from an aws cli skeleton")
    .arguments("<awsService:string> <awsCommand:string>")
    .option("--input <input:string>", "awsInputPath:siPropertiesPath; constructs CLI json data", { collect: true, required: true })
    .action(async (options, awsService, awsCommand) => {
      const deleteOptions = makeDeleteOrActionOptions(options);
      const result = await renderDelete({ provider: "aws", awsService, awsCommand, inputs: deleteOptions.inputs });
      console.log(result);
    })
    .command("action")
    .description("generate an action function from an aws cli skeleton")
    .arguments("<awsService:string> <awsCommand:string>")
    .option("--input <input:string>", "awsInputPath:siPropertiesPath; constructs CLI json data", { collect: true, required: true })
    .action(async (options, awsService, awsCommand) => {
      const deleteOptions = makeDeleteOrActionOptions(options);
      const result = await renderAction({ provider: "aws", awsService, awsCommand, inputs: deleteOptions.inputs });
      console.log(result);
    });

  const _result = await command.parse(Deno.args);
}
