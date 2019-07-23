import { AuthenticationError } from 'apollo-server';

import { GraphQLModule } from '@graphql-modules/core';
import gql from 'graphql-tag';

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
      testMessage: (_obj, _args, context, info) => {
        console.log(info.session.req.user);
        if (info.session.req.user) {
          return 'logged in';
        } else {
          return 'not logged in';
        }
      },
    },
  },
});
