import OpenAI from "https://deno.land/x/openai@v4.67.1/mod.ts";

async function main() {
  const openai = new OpenAI();
  const storeList = await openai.beta.vectorStores.list();
  let vectorStore;
  for (const store of storeList.data) {
    if (store.name == "System Initiative Schema") {
      vectorStore = store;
    }
  }
  if (!vectorStore) {
    const vectorStore = await openai.beta.vectorStores.create({
      name: "System Initiative Schema",
    });
  }
}

await main();
