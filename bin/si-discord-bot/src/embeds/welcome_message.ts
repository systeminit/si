import { MessageEmbed } from "discord.js";
import { getRoleMention } from "../roles";
import { getChannelMention } from "../channels";

export function welcomeMessage(): MessageEmbed {
  const welcomeMessage = new MessageEmbed().setTitle(
    "Welcome to the System Initiative Discord Server!"
  )
    .setDescription(`We're glad you've joined us! We talk about all things related to DevOps, cloud computing, software development, and other nerdy topics.
    
    Please go to the **${getChannelMention("rules")}** channel, read over our community rules, and click on the ✅ to agree to them and gain access to our community.
  
  If you need help, let any member of ${getRoleMention("Team SI")} know.
`);
  return welcomeMessage;
}
