import { GraphQLModule } from '@graphql-modules/core';
import gql from "graphql-tag";

export const HelloWorld = new GraphQLModule({
  typeDefs: gql`
    type Query {
      """
      test message
      """
      testMessage: String!
    }
  `,
  resolvers: {
    Query: {
      testMessage: (): string => {
        return "foo";
      },
    }
  }
});
