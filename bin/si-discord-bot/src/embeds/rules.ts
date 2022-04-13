import { MessageEmbed } from "discord.js";
import { getRoleMention } from "../roles";

export function rulesMessage(): MessageEmbed {
  const rulesMessage = new MessageEmbed()
    .setTitle(
      "System Initiative Community Guidlines"
    )
    .setDescription(`
These are the rules:

1. *Follow the code of conduct*:
Please follow [Rust's Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).
Report any violations or issues to an ${getRoleMention("Admin")}.

2. *New channels*:
We're happy to have new public channels on any topic related to cloud computing. Let a member of ${getRoleMention("Team SI")} know what you would like, and we'll take care of the rest.
`);
  return rulesMessage;
}

