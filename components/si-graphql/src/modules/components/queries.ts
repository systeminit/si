import { UserInputError } from "apollo-server";
import hasIn from "lodash/hasIn";
import find from "lodash/find";
import filter from "lodash/filter";
import { matchArray, SearchPrimitive } from "searchjs";

import { checkAuthentication } from "@/modules/auth";
import { GqlRoot, GqlContext, GqlInfo } from "@/app.module";
import { getServerComponents } from "@/modules/servers/queries";
import { getOperatingSystemComponents } from "@/modules/operating-systems/queries";
import { getDiskImageComponents } from "@/modules/disk-images/queries";
import { getCpuComponents } from "@/modules/cpus/queries";
import { getPortComponents } from "@/modules/ports/queries";
import { getSshKeyComponents } from "@/modules/ssh-key/queries";
import { User } from "@/datalayer/user";
import { Workspace } from "@/datalayer/workspace";
import { ComponentObject } from "@/datalayer/component";

export interface GetComponentsInput {
  where?: {
    integration?: string;
    workspace?: string;
  };
}

export interface FindComponentInput {
  where: {
    workspace?: string;
    search: string;
  };
}

export async function getComponents(
  obj: GqlRoot,
  args: GetComponentsInput,
  context: GqlContext,
  info: GqlInfo,
): Promise<ComponentObject[]> {
  await checkAuthentication(info);
  let data: ComponentObject[] = [];
  const serverData = await getServerComponents(obj, args, context, info);
  const osData = await getOperatingSystemComponents(obj, args, context, info);
  const imageData = await getDiskImageComponents(obj, args, context, info);
  const cpuData = await getCpuComponents(obj, args, context, info);
  const portData = await getPortComponents(obj, args, context, info);
  const sshKeyData = await getSshKeyComponents(obj, args, context, info);
  data = data.concat(serverData);
  data = data.concat(osData);
  data = data.concat(imageData);
  data = data.concat(cpuData);
  data = data.concat(portData);
  data = data.concat(sshKeyData);
  return data;
}

export async function filterComponents<T extends ComponentObject>(
  data: T[],
  args: GetComponentsInput,
  user: User,
): Promise<T[]> {
  // Limit by Workspace
  if (hasIn(args, "where.workspace")) {
    const workspaces = await Workspace.getWorkspacesForUser(user);
    const filterWorkspace = find(workspaces, { id: args.where.workspace });
    if (filterWorkspace === undefined) {
      console.log(workspaces);
      console.log(filterWorkspace);
      throw new UserInputError("Workspace is not associated with the user", {
        invalidArgs: ["workspace"],
      });
    }

    const integrationInstances = await filterWorkspace.integrationInstances();

    const result = filter(data, (o: T): boolean => {
      for (const integrationInstance of integrationInstances) {
        if (o.integrationId == integrationInstance.integrationId) {
          return true;
        }
      }
      // The magic guid for the global integration
      if (
        o.integrationId == "integration:9bfc0c3e-6273-4196-8e74-364761cb1b04"
      ) {
        return true;
      } else {
        return false;
      }
    });
    return result;
    // Limit by Integration
  } else if (hasIn(args, "where.integration")) {
    const result = filter(data, (o: T): boolean => {
      return o.integrationId == `integration:${args.where.integration}`;
    });
    return result;
  } else {
    return data;
  }
}

export async function findComponents(
  obj: GqlRoot,
  args: FindComponentInput,
  context: GqlContext,
  info: GqlInfo,
): Promise<ComponentObject[]> {
  let data: ComponentObject[];

  if (args.where.workspace) {
    data = await getComponents(
      obj,
      { where: { workspace: args.where.workspace } },
      context,
      info,
    );
  } else {
    data = await getComponents(obj, {}, context, info);
  }
  const searchArguments = JSON.parse(args.where.search) as SearchPrimitive;
  return matchArray(data, searchArguments);
}
