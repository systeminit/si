import { MessageEmbed, } from "discord.js";
import { CHANNELS } from "./channels";
import { rulesMessage } from "./embeds/rules";
import { welcomeMessage } from "./embeds/welcome_message";

export async function syncEmbeds() {
  await sendEmbed("welcome", welcomeMessage());
  await sendEmbed("rules", rulesMessage());
}

export async function sendEmbed(
  channelName: string,
  embedToSend: MessageEmbed
): Promise<boolean> {
  const channel = CHANNELS[channelName];
  if (channel) {
    let embedSent = false;
    const messages = await channel.messages.fetch();
    for (const msg of messages.values()) {
      for (const embed of msg.embeds.values()) {
        if (embed.title == embedToSend.title) {
          msg.edit({ embeds: [embedToSend] });
          embedSent = true;
        }
      }
    }
    if (!embedSent) {
      channel.send({ embeds: [embedToSend] });
    }
  } else {
    throw new Error(`cannot send message to missing channel: ${channelName} `)
  }

  return true;
}
