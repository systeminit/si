import { User } from "@/datalayer/user";
import { GqlRoot, GqlContext, GqlInfo } from "@/app.module";

interface CreateUserResponse {
  user: User;
}

export async function createUser(
  _obj: GqlRoot,
  { input: { email, name } },
  _context: GqlContext,
  _info: GqlInfo,
): Promise<CreateUserResponse> {
  const user = await User.createOrReturn({ email, name });
  return {
    user,
  };
}
