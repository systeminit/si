import { GraphQLModule } from "@graphql-modules/core";
import { HelloWorld } from "@modules/hello-world";
import { Users } from "@modules/users";
import { Workspaces } from "@modules/workspaces";
import { Integrations } from "@modules/integrations";
import { Components } from "@modules/components";
import { Servers } from "@modules/servers";

export interface GqlRoot {
  [key: string]: any; //eslint-disable-line
}

export interface GqlArgs {
  [key: string]: any; //eslint-disable-line
}

export interface GqlContext {
  [key: string]: any; //eslint-disable-line
}

export interface GqlInfo {
  [key: string]: any; //eslint-disable-line
}

export const AppModule = new GraphQLModule({
  imports: [HelloWorld, Users, Workspaces, Integrations, Components, Servers],
});
