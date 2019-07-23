import { AuthenticationError } from "apollo-server";

import { User } from "@/datalayer/user";

export async function checkAuthentication(info): Promise<User> {
  if (info.session.req.user === undefined) {
    throw new AuthenticationError("must authenticate");
  }

  // If we are authenticated, then we should create the record in
  // the database. We know we will need it later.
  return await User.createOrReturn(
    info.session.req.user.email,
    info.session.req.user.name,
  );
}
