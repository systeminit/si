import { registry } from "si-registry";
import { apollo } from "./apollo";

export async function login(
  billingAccountName: string,
  email: string,
  password: string,
): Promise<Record<string, any>> {
  if (!localStorage.getItem("apollo-token")) {
    const user = registry.get("user");
    const loginQuery = user.graphql.query({
      methodName: "loginInternal",
      overrideName: "userLogin",
      overrideFields: "jwt, userId, billingAccountId",
    });
    console.log("I am about to log in");
    const loginResult = await apollo.query({
      query: loginQuery,
      variables: { billingAccountName, email, password },
    });
    // This configures apollo to use this token for all requests, if it exists.
    localStorage.setItem("apollo-token", loginResult.data.userLogin.jwt);

    console.log("I logged in! So cool", { loginResult });

    // Set up the users profile
    const userQuery = user.graphql.query({
      methodName: "get",
      associations: {
        user: ["billingAccount"],
        billingAccount: ["organizations"],
        organization: ["workspaces"],
      },
    });
    const userReply = await apollo.query({
      query: userQuery,
      variables: { id: loginResult.data.userLogin.userId },
    });
    const data = user.graphql.validateResult({
      methodName: "get",
      data: userReply,
    });
    const profile = {
      user: data.item,
      billingAccount: data.item.associations.billingAccount.item,
      organization:
        data.item.associations.billingAccount.item.associations.organizations
          .items[0],
      workspaces:
        data.item.associations.billingAccount.item.associations.organizations
          .items[0].associations.workspaces.items,
      workspaceDefault:
        data.item.associations.billingAccount.item.associations.organizations
          .items[0].associations.workspaces.items[0],
    };
    localStorage.setItem("profile", JSON.stringify(profile));
    return profile;
  } else {
    const profile = localStorage.getItem("profile");
    if (profile) {
      return JSON.parse(profile);
    } else {
      throw "Cannot deserialize profile, even though apollo-token exists";
    }
  }
}

export async function loginBobo(): Promise<Record<string, any>> {
  return login("boboCorp", "bobo@bobotclown.co", "bobotclown42");
}

export async function logout(): Promise<boolean> {
  localStorage.removeItem("profile");
  localStorage.removeItem("apollo-token");
  return true;
}
