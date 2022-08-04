import { MessageEmbed } from "discord.js";
import { getRoleMention } from "../roles";
import { getChannelMention } from "../channels";

export function welcomeMessage(): MessageEmbed {
  const welcomeMessage = new MessageEmbed().setTitle(
    "Welcome to the System Initiative Discord Server"
  )
    .setDescription(`We're glad you've joined us! Take a minute to familiarize yourself with the ${getChannelMention(
    "rules"
  )}, then head over to ${getChannelMention(
    "get-roles"
  )} to tell us a little more about yourself. 

We talk about all things related to cloud computing, software development, and other nerdy topics in ${getChannelMention(
    "general"
  )} and other channels in the "💬 PUBLIC 💬" category.

If you've been invited to become an ${getRoleMention(
    "Agent"
  )} - let us know who you are in ${getChannelMention("general")}.

If you need help, let any member of ${getRoleMention("Team SI")} know.
`);
  return welcomeMessage;
}
