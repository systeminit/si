import { roleMention } from "@discordjs/builders";
import { Client, Role } from "discord.js";

interface Roles {
  [key: string]: Role,
}

export const ROLES: Roles = {};

export async function loadRoles(client: Client) {
  for (const guild of client.guilds.cache.values()) {
    if (guild.name == "System Initiative") {
      for (const role of guild.roles.cache.values()) {
        ROLES[role.name] = role;
      }
    }
  }
  //console.log(ROLES);
}

export function getRoleMention(role: string): string {
  if (ROLES[role]) {
    return roleMention(ROLES[role].id);
  } else {
    throw new Error(`cannot find role: ${role}`);
  }
}
