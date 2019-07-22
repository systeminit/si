import { ApolloServer } from 'apollo-server';

import { environment } from './environment';

import { HelloWorld } from '@modules/hello-world';

const server = new ApolloServer({ 
  context: session => session,
  modules: [
    HelloWorld
  ],
  //resolvers, 
  //typeDefs,
  introspection: environment.apollo.introspection,
  playground: environment.apollo.playground
});

server.listen(environment.port)
  .then(({ url }) => console.log(`Server ready at ${url}. `));

// Hot Module Replacement
if (module.hot) {
  module.hot.accept();
  module.hot.dispose(() => server.stop());
}
