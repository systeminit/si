import { AuthenticationError } from "apollo-server";

import { User } from "@/datalayer/user";
import { GqlInfo } from "@/app.module";

export async function checkAuthentication(info: GqlInfo): Promise<User> {
  if (info.session.req.user === undefined) {
    throw new AuthenticationError("must authenticate");
  }

  // If we are authenticated, then we should create the record in
  // the database. We know we will need it later.
  return User.createOrReturn({
    email: info.session.req.user.email,
    name: info.session.req.user.name,
  });
}
