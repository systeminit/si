import { checkAuthentication } from "@/modules/auth";
import {
  ServerComponent,
  serverComponentData,
} from "@/datalayer/serverComponents";
import { GqlRoot, GqlContext, GqlInfo } from "@/app.modules";
import {
  GetComponentsInput,
  filterComponents,
} from "@/modules/components/queries";

export async function getServerComponents(
  _obj: GqlRoot,
  args: GetComponentsInput,
  _context: GqlContext,
  info: GqlInfo,
): Promise<ServerComponent[]> {
  let user = await checkAuthentication(info);
  let data: ServerComponent[] = await serverComponentData();
  return filterComponents(data, args, user);

  // Limit by Workspace
  //if (hasIn(args, "where.workspace")) {
  //  let workspace: Workspace[] = await user
  //    .$relatedQuery("workspaces")
  //    .where("id", args.where.workspace);
  //  if (workspace.length == 0) {
  //    throw new UserInputError("Workspace is not associated with the user", {
  //      invalidArgs: ["workspace"],
  //    });
  //  }
  //  let integrationInstances: IntegrationInstance[] = await workspace[0]
  //    .$relatedQuery("integrationInstances")
  //    .eager("integration");
  //  let result = filter(data, (o: ServerComponent): boolean => {
  //    for (let x = 0; x < integrationInstances.length; x++) {
  //      if (integrationInstances[x].integration.id == o.integration.id) {
  //        return true;
  //      }
  //    }
  //    return false;
  //  });
  //  return result;
  //  // Limit by Integration
  //} else if (hasIn(args, "where.integration")) {
  //  let result = filter(data, (o: ServerComponent): boolean => {
  //    return `${o.integration.id}` == args.where.integration;
  //  });
  //  return result;
  //} else {
  //  return data;
  //}
}
