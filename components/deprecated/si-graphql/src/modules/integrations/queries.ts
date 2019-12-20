import { GqlRoot, GqlArgs, GqlContext, GqlInfo } from "@/app.module";
import { Integration, IntegrationInstance } from "@/datalayer/integration";
import { checkAuthentication } from "@/modules/auth";

export async function getAllIntegrations(
  _obj: GqlRoot,
  _input: GqlArgs,
  _context: GqlContext,
  info: GqlInfo,
): Promise<Integration[]> {
  await checkAuthentication(info);
  return Integration.getAll();
}

export async function getIntegrationInstances(
  _obj: GqlRoot,
  _input: GqlArgs,
  _context: GqlContext,
  info: GqlInfo,
): Promise<IntegrationInstance[]> {
  const currentUser = await checkAuthentication(info);
  return IntegrationInstance.getForUser(currentUser);
}

export async function getIntegrationInstanceById(
  _obj: GqlRoot,
  { input: { id: id } },
  _context: GqlContext,
  info: GqlInfo,
): Promise<IntegrationInstance> {
  const user = await checkAuthentication(info);
  return IntegrationInstance.getById(id, user);
}
