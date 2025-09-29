import adze, { setup } from "adze";

const activeLevel = Deno.env.get("LOG_LEVEL") ?? "info";

setup({
  // @ts-ignore Yeah yeah, it's okay - we know they could use a bad level
  activeLevel,
  meta: {
    "si": "is fun",
  },
});

const logger = adze.withEmoji.timestamp.seal();
export default logger;
