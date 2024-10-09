import OpenAI from "https://deno.land/x/openai@v4.67.1/mod.ts";

// Helper function to convert FsFile to File
async function convertToFile(path: string): Promise<File> {
  const fileContent = await Deno.readFile(path); // Read file contents
  const fileName = path.split('/').pop() || 'unknown'; // Extract file name from path
  return new File([fileContent], fileName, { type: "application/octet-stream" });
}

async function main() {
  const openai = new OpenAI();
  const systemContent = await Deno.readTextFile("./prompts/1-system.txt");

  // Convert files to File objects
  const filePaths = [
    "./data/aws-cli.pdf",
    "../../app/docs/src/reference/asset/schema.md",
  ];

  const fileList = await Promise.all(filePaths.map(convertToFile));

  const response = await openai.beta.vectorStores.list();
  for (const store of response.data) {
    if (store.name == "System Initiative Schema") {

    }
  }

  const vectorStore = await openai.beta.vectorStores.create({
    name: "System Initiative Schema",
  });

  const assistant = await openai.beta.assistants.create({
    name: "System Initiative Schema Builder",
    instructions: systemContent,
    model: "gpt-4o",
    tools: [{ type: "file_search" }],
    tool_resources: { file_search: { vector_store_ids: [vectorStore.id] } },
  });

  const userQuestion = {
    role: "user",
    content: "Create an asset for an AWS Lambda Function",
  };

  const thread = await openai.beta.threads.create({
    messages: [userQuestion],
  });

  const run = await openai.beta.threads.runs.createAndPoll(thread.id, {
    assistant_id: assistant.id,
  });

  const messages = await openai.beta.threads.messages.list(thread.id, { run_id: run.id });
  console.log(messages);
}

await main();

