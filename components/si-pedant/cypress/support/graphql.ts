import { registry } from "si-registry";
import { apollo } from "./apollo";
import { ApolloQueryResult } from "apollo-client";
import { FetchResult } from "apollo-link";

export function graphqlQuery(
  args: GraphqlQueryArgs,
): Promise<ApolloQueryResult<Record<string, any>>> {
  const getValue = () => {
    const siObject = registry.get(args.typeName);
    const query = siObject.graphql.query(args.queryArgs);

    return apollo
      .query({
        query,
        variables: args.variables,
      })
      .then((result) => {
        return result;
      });
  };
  const resolveValue = () => {
    return Cypress.Promise.try(getValue).then((value) => {
      return cy.verifyUpcomingAssertions(value, args, {
        onRetry: resolveValue,
      });
    });
  };
  return resolveValue();
}

export function graphqlMutation(args: GraphqlQueryArgs): Promise<FetchResult> {
  const getValue = () => {
    const siObject = registry.get(args.typeName);
    const mutation = siObject.graphql.mutation(args.queryArgs);
    console.log("failed here");
    return apollo
      .mutate({
        mutation,
        variables: args.variables,
      })
      .then((result) => {
        return result;
      });
  };
  const resolveValue = () => {
    return Cypress.Promise.try(getValue).then((value) => {
      return value;
    });
  };
  return resolveValue();
}
