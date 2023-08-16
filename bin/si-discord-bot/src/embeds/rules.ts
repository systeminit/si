import { MessageEmbed } from "discord.js";
import { getRoleMention } from "../roles";
import { getChannelMention } from "../channels";

export function rulesMessage(): MessageEmbed {
  const rulesMessage = new MessageEmbed().setTitle(
    "System Initiative Community Guidlines"
  ).setDescription(`
These are the rules:

***1. Code of Conduct:***
Please follow **[Our Code of Conduct](https://github.com/systeminit/si/blob/main/CODE_OF_CONDUCT.md)**.
Report any violations or issues to an ${getRoleMention("Admin")}.

We strive to make our community a welcoming, inclusive, and positive space, and reserve the right to remove any community member who impedes that goal.

***2. Nicknames:***
Please set your server nickname to the name you would like other members of our community to call you.

***3. Roles:***
Please select roles that apply to you in the ${getChannelMention("get-roles")} channel so that other community members can know more about you.

***4. Channels:***
We talk about all things related to DevOps, cloud computing, software development, and other nerdy topics in general in the ***💬 PUBLIC 💬*** category. Please keep channels on topic. If you would like to suggest a channel for a new topic, let a member of ${getRoleMention("Team SI")} know.

Click the ✅ below this post to agree to these rules and gain access to our community.
`);
  return rulesMessage;
}
