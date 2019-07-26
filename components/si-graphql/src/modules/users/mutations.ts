import { User } from '@/datalayer/user';

interface CreateUserResponse {
  user: User;
}

export async function createUser(
  _obj,
  args,
  _context,
  _info,
): Promise<CreateUserResponse> {
  const user = await User.createOrReturn(args.input.email, args.input.name);
  return {
    user,
  };
}
