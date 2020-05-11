import { SiGraphql } from "./graphql";
import { registry } from "../registry";
import "../loader";

test("query generation", done => {
  const systemObject = registry.get("billingAccount");
  const siGraphql = new SiGraphql(systemObject);
  const queryResult = siGraphql.query({ methodName: "get" });
  // We want to match associations based on type and field name. Whenever we
  // see a type, we check the associations for a matching entry, and if we
  // find it, we run the list of field names to load, and then we load them.
  const user = registry.get("user");
  const userQuery = user.graphql.query({
    methodName: "get",
    associations: {
      user: ["billingAccount"],
      billingAccount: ["organizations"],
      organization: ["workspaces"],
    },
  });

  done();
});
