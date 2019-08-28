import { User } from "@/datalayer/user";
import { checkAuthentication } from "@/modules/auth";
import { GqlRoot, GqlContext, GqlInfo } from "@/app.module";

// This function is probably a bad idea; it lets you get the user
// information for any user at all. Eventually, be a good idea to
// wrap some authorization, or remove this function entirely from the
// graphql api.
export async function getUserById(
  _obj: GqlRoot,
  { input: { id: id } },
  _context: GqlContext,
  info: GqlInfo,
): Promise<User> {
  checkAuthentication(info);
  return User.getById(id);
}
