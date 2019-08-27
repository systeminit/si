import { UserInputError } from "apollo-server";
import hasIn from "lodash/hasIn";
import filter from "lodash/filter";

import { checkAuthentication } from "@/modules/auth";
import { IntegrationInstance } from "@/datalayer/integration";
import { GqlRoot, GqlContext, GqlInfo } from "@/app.module";
import { getServerComponents } from "@/modules/servers/queries";
import { getOperatingSystemComponents } from "@/modules/operating-systems/queries";
import { User } from "@/datalayer/user";
import { Workspace } from "@/datalayer/workspace";
import { Component } from "@/datalayer/component";

export interface GetComponentsInput {
  where?: {
    integration?: string;
    workspace?: string;
  };
}

export async function getComponents(
  obj: GqlRoot,
  args: GetComponentsInput,
  context: GqlContext,
  info: GqlInfo,
): Promise<Component[]> {
  await checkAuthentication(info);
  let data: Component[] = [];
  const serverData = await getServerComponents(obj, args, context, info);
  const osData = await getOperatingSystemComponents(obj, args, context, info);
  data = data.concat(serverData);
  data = data.concat(osData);
  return data;
}

export async function filterComponents<T extends Component>(
  data: T[],
  args: GetComponentsInput,
  user: User,
): Promise<T[]> {
  // Limit by Workspace
  if (hasIn(args, "where.workspace")) {
    const workspace: Workspace[] = await user
      .$relatedQuery("workspaces")
      .where("id", args.where.workspace);
    if (workspace.length == 0) {
      throw new UserInputError("Workspace is not associated with the user", {
        invalidArgs: ["workspace"],
      });
    }
    const integrationInstances: IntegrationInstance[] = await workspace[0]
      .$relatedQuery("integrationInstances")
      .eager("integration");
    const result = filter(data, (o: T): boolean => {
      for (let x = 0; x < integrationInstances.length; x++) {
        // HACK: 5 is the magic number of the global integration
        if (
          integrationInstances[x].integration.id == o.integration.id ||
          o.integration.id == "9bfc0c3e-6273-4196-8e74-364761cb1b04" // The magic guid for the global integration
        ) {
          return true;
        }
      }
      return false;
    });
    return result;
    // Limit by Integration
  } else if (hasIn(args, "where.integration")) {
    const result = filter(data, (o: T): boolean => {
      return `${o.integration.id}` == args.where.integration;
    });
    return result;
  } else {
    return data;
  }
}
