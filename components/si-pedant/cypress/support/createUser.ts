import { registry } from "si-registry";
import { apollo } from "./apollo";

export async function createUser(args: CreateUserArgs): Promise<boolean> {
  const billingAccount = registry.get("billingAccount");
  const signupMutation = billingAccount.graphql.mutation({
    methodName: "signup",
  });

  try {
    const signupResult = await apollo.mutate({
      mutation: signupMutation,
      variables: args,
    });
    console.log("signup Result", { signupResult });
  } catch (err) {
    if (err.message.includes("already exists")) {
      return true;
    }
    console.log(err);
    throw err;
  }
  return true;
}

export async function createUserBobo(): Promise<boolean> {
  return createUser({
    billingAccount: {
      name: "boboCorp",
      displayName: "Bobo Corp",
    },
    user: {
      name: "bobo",
      displayName: "Bobo T. Clown",
      email: "bobo@bobotclown.co",
      password: "bobotclown42",
    },
  });
}
