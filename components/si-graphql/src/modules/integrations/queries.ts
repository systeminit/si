import { Integration, IntegrationInstance } from "@/datalayer/integration";
import { checkAuthentication } from "@/modules/auth";

export async function getAllIntegrations(
  _obj,
  _input,
  _context,
  info,
): Promise<Integration[]> {
  await checkAuthentication(info);
  return Integration.query();
}

export async function getIntegrationInstances(
  _obj,
  _input,
  _context,
  info,
): Promise<Integration[]> {
  const currentUser = await checkAuthentication(info);
  return IntegrationInstance.getIntegrationInstances(currentUser);
}

export async function getIntegrationInstanceById(
  _obj,
  { input: { id: id } },
  _context,
  info,
): Promise<IntegrationInstance> {
  const user = await checkAuthentication(info);
  return IntegrationInstance.query()
    .where("user_id", user.id)
    .findById(id);
}
