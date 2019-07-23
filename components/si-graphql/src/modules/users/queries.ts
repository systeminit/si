import { User } from "@/datalayer/user";
import { checkAuthentication } from "@/modules/auth";

// This function is probably a bad idea; it lets you get the user
// information for any user at all. Eventually, be a good idea to
// wrap some authorization, or remove this function entirely from the
// graphql api.
export async function getUserById(
  _obj,
  { input: { id: id } },
  _context,
  info,
): Promise<User> {
  await checkAuthentication(info);
  return User.query().findById(id);
}
