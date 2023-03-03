import { MessageEmbed } from "discord.js";
import { getRoleMention } from "../roles";
import { getChannelMention } from "../channels";

export function changeoverMessage(): MessageEmbed {
  const changeoverMessage = new MessageEmbed().setTitle(
    "System Initiative Community Guidlines"
  ).setDescription(`Hello @everyone!

  We will soon be making a change to server permissions to ensure that this space remains a welcoming, positive, and inclusive one for our Discord community. The change is simple: in order to view or post in the server, members will need to agree  to the server **${getChannelMention("rules")}**. This change will go into effect on **Sunday March 5th at 9pm PT.**
  
  If you have not already done so, please go read over the rules in the **${getChannelMention("rules")}** channel and click on the ✅ to be granted the ${getRoleMention("Members")} role and retain access to the Discord. If you do not do so before we make this change, you will be locked out of most of the channels until you do so.
  
  Thank you all for being a part of our awesome community!
  `);
  return changeoverMessage;
}
