import { channelMention } from "@discordjs/builders";
import { Client, TextChannel } from "discord.js";

interface TextChannels {
  [key: string]: TextChannel;
}

export const CHANNELS: TextChannels = {};

export async function loadChannels(client: Client) {
  for (const channel of client.channels.cache.values()) {
    //console.log(channel);
    if (channel.type == "GUILD_TEXT") {
      CHANNELS[channel.name] = channel;
    }
  }
  //console.log(CHANNELS);
}

export function getChannelMention(channel: string): string {
  if (CHANNELS[channel]) {
    return channelMention(CHANNELS[channel].id);
  } else {
    //console.log(CHANNELS)
    throw new Error(`cannot find channel: ${channel}`);
  }
}
