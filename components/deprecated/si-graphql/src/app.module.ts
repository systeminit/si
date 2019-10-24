import { GraphQLModule } from "@graphql-modules/core";
import { HelloWorld } from "@modules/hello-world";
import { Users } from "@modules/users";
import { Workspaces } from "@modules/workspaces";
import { Integrations } from "@modules/integrations";
import { Components } from "@modules/components";
import { Servers } from "@modules/servers";
import { OperatingSystems } from "@modules/operating-systems";
import { DiskImages } from "@modules/disk-images";
import { Ports } from "@modules/ports";
import { Cpus } from "@modules/cpus";
import { SshKeys } from "@modules/ssh-key";

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

// Make the component relationships work
import "@/datalayer/component/relationships";

export const AppModule = new GraphQLModule({
  imports: [
    HelloWorld,
    Users,
    Workspaces,
    Integrations,
    Components,
    Servers,
    OperatingSystems,
    DiskImages,
    Cpus,
    Ports,
    SshKeys,
  ],
});
