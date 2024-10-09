import { ChatCompletionUserMessageParam } from "https://deno.land/x/openai@v4.67.1/resources/chat/completions.ts";

const assetDocsPreamble =
  "What follows is the current System Initiative schema API documentation, between three ^ characters.\n^^^\n";
const assetDocs = await Deno.readTextFile(
  "../../app/docs/src/reference/asset/schema.md",
);
const assetDocsEnd = "\n^^^\n";

export const assetSchemaPrompt: ChatCompletionUserMessageParam = {
  role: "user",
  content: assetDocsPreamble + assetDocs + assetDocsEnd,
};

export const createAssetPrompt: ChatCompletionUserMessageParam = {
  role: "user",
  content: "Create the asset",
};

const functionDocsPreamble = "What follows is the current System Initiative function API documentation, between three ^ characters.\n^^^\n";
const functionDocs = await Deno.readTextFile("../../app/docs/src/reference/asset/function.md");
const functionDocsEnd = "\n^^^\n";

export const functionPrompt: ChatCompletionUserMessageParam = {
  role: "user",
  content: functionDocsPreamble + functionDocs + functionDocsEnd,
};

export const createFunctionPropmt: ChatCompletionUserMessageParam = {
  role: "user",
  content: "Create the function",
};


