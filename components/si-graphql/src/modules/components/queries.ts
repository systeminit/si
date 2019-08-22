import { UserInputError } from "apollo-server";
import hasIn from "lodash/hasIn";
import filter from "lodash/filter";

import { checkAuthentication } from "@/modules/auth";
import { ServerComponent } from "@/datalayer/serverComponents";
import { Integration } from "@/datalayer/integration";
import { GqlRoot, GqlContext, GqlInfo } from "@/app.modules";
import { getServerComponents } from "@/modules/servers/queries";
import { User } from "@/datalayer/user";
import { Workspace } from "@/datalayer/workspace";
import { IntegrationInstance } from "@/datalayer/integration";

export interface Component {
  id: string;
  name: string;
  description: string;
  rawDataJson: string;
  integration: Integration;
  memoryGIB: number;
  nodeType: string;
}

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
  let serverData: ServerComponent[] = await getServerComponents(
    obj,
    args,
    context,
    info,
  );
  data = data.concat(serverData);
  return data;
}

export async function filterComponents<T extends Component>(
  data: T[],
  args: GetComponentsInput,
  user: User,
): Promise<T[]> {
  // Limit by Workspace
  if (hasIn(args, "where.workspace")) {
    let workspace: Workspace[] = await user
      .$relatedQuery("workspaces")
      .where("id", args.where.workspace);
    if (workspace.length == 0) {
      throw new UserInputError("Workspace is not associated with the user", {
        invalidArgs: ["workspace"],
      });
    }
    let integrationInstances: IntegrationInstance[] = await workspace[0]
      .$relatedQuery("integrationInstances")
      .eager("integration");
    let result = filter(data, (o: T): boolean => {
      for (let x = 0; x < integrationInstances.length; x++) {
        if (integrationInstances[x].integration.id == o.integration.id) {
          return true;
        }
      }
      return false;
    });
    return result;
    // Limit by Integration
  } else if (hasIn(args, "where.integration")) {
    let result = filter(data, (o: T): boolean => {
      return `${o.integration.id}` == args.where.integration;
    });
    return result;
  } else {
    return data;
  }
}
