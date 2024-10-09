import OpenAI from "https://deno.land/x/openai@v4.67.1/mod.ts";
import { commandWithPager } from "./readManPage.ts";
import { assetSchemaPrompt, createAssetPrompt } from "./si.ts";
import {
  ChatCompletionSystemMessageParam,
  ChatCompletionUserMessageParam,
} from "https://deno.land/x/openai@v4.67.1/resources/chat/completions.ts";
import { readWebPage } from "./readWebPage.ts";
import { functionPrompt } from "./si.ts";
import { createFunctionPropmt } from "./si.ts";

async function readManPage(section: string, call: string): Promise<string> {
  return commandWithPager("aws", [section, call, "help"]);
}

export async function generateAwsSchema(
  awsService: string,
  awsCommand: string,
  propsFromUrl: { property: string; url: string }[],
): Promise<string> {
  const openai = new OpenAI();
  const systemContent = await Deno.readTextFile(
    "./src/prompts/system-aws-asset.txt",
  );
  const systemPrompt: ChatCompletionSystemMessageParam = {
    role: "system",
    content: systemContent,
  };

  const awsCommandPreamble =
    "What follows is the help output from an AWS CLI command as ASCII text, between three ^ characters.\n^^^\n";
  const awsCommandData = await readManPage(awsService, awsCommand);
  const awsDocsEnd = "\n^^^\n";
  const awsData: ChatCompletionUserMessageParam = {
    role: "user",
    content: awsCommandPreamble + awsCommandData + awsDocsEnd,
  };

  const messages = [
    systemPrompt,
    assetSchemaPrompt,
  ];

  for (const propFromUrl of propsFromUrl) {
    console.log("loading", propFromUrl);
    const page = readWebPage(propFromUrl.url);
    console.log(page);
    const preamble =
      `Use the documentation found between three ^ characters to build the property entries for the ${propFromUrl.property} property and its children, if any, rather than any other description for this property. If the documentation is JSON, it is in JSON schema format, otherwise it is a webpage.\n^^^\n`;
    const propData: ChatCompletionUserMessageParam = {
      role: "user",
      content: preamble + page + "\n^^^\n",
    };
    awsData.content = awsData.content + preamble + page + "\n^^^\n";
  }

  messages.push(awsData);
  messages.push(createAssetPrompt);
  console.log("firing up");
  const completion = await openai.chat.completions.create({
    model: "gpt-4o",
    messages,
    temperature: 0,
  });
  console.log(completion);
  return completion.choices[0].message.content || "bad result";
}

export async function generateAwsFunction(
  awsService: string,
  awsCommand: string,
  propsFromUrl: { property: string; url: string }[],
): Promise<string> {
  const openai = new OpenAI();
  const systemContent = await Deno.readTextFile(
    "./src/prompts/system-aws-action.txt",
  );
  const systemPrompt: ChatCompletionSystemMessageParam = {
    role: "system",
    content: systemContent,
  };

  const awsCommandPreamble =
    "What follows is the help output from an AWS CLI command as ASCII text, between three ^ characters.\n^^^\n";
  const awsCommandData = await readManPage(awsService, awsCommand);
  const awsDocsEnd = "\n^^^\n";
  const awsData: ChatCompletionUserMessageParam = {
    role: "user",
    content: awsCommandPreamble + awsCommandData + awsDocsEnd,
  };

  const messages = [
    systemPrompt,
    assetSchemaPrompt,
    functionPrompt,
  ];

  messages.push(awsData);
  messages.push(createFunctionPropmt);
  console.log("firing up");
  const completion = await openai.chat.completions.create({
    model: "gpt-4o",
    messages,
    temperature: 0,
  });
  console.log(completion);
  return completion.choices[0].message.content || "bad result";
}

