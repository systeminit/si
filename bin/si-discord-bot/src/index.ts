#!/usr/bin/env tsc

// Require the necessary discord.js classes
import { Client, Intents }  from "discord.js";
import { loadChannels } from "./channels";
import { syncEmbeds } from "./embeds";
import { loadRoles, } from "./roles";

const TOKEN = process.env["DISCORD_TOKEN"];

async function main() {
  console.log("starting");
  // Create a new client instance
  const client = new Client({
    intents: [
      Intents.FLAGS.GUILDS,
      Intents.FLAGS.GUILD_MESSAGES,
      Intents.FLAGS.GUILD_MESSAGE_REACTIONS,
      Intents.FLAGS.GUILD_MESSAGE_TYPING,
      Intents.FLAGS.GUILD_MEMBERS,
      Intents.FLAGS.DIRECT_MESSAGES,
      Intents.FLAGS.DIRECT_MESSAGE_REACTIONS,
      Intents.FLAGS.DIRECT_MESSAGE_TYPING,
    ],
    partials: ["MESSAGE", "CHANNEL"],
  });

  // When the client is ready, run this code (only once)
  client.once("ready", async () => {
    console.log("loading...");
    await loadRoles(client);
    await loadChannels(client);
    await syncEmbeds();
    console.log("ready");
  });

  console.log("logging in...");
  // Login to Discord with your client's token
  client.login(TOKEN);
}

main();
