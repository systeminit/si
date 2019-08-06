import express from "express";
import { ApolloServer } from "apollo-server-express";
import jwt from "express-jwt";
import jwksRsa from "jwks-rsa";

import { HelloWorld } from "@modules/hello-world";
import { Users } from "@modules/users";
import { Workspaces } from "@modules/workspaces";
import { Integrations } from "@modules/integrations";
import { environment } from "@/environment";

// Authentication middleware. When used, the
// Access Token must exist and be verified against
// the Auth0 JSON Web Key Set
const checkJwt = jwt({
  // Dynamically provide a signing key
  // based on the kid in the header and
  // the signing keys provided by the JWKS endpoint.
  secret: jwksRsa.expressJwtSecret({
    cache: true,
    rateLimit: true,
    jwksRequestsPerMinute: 5,
    jwksUri: `https://systeminit.auth0.com/.well-known/jwks.json`,
  }),

  // Validate the audience and the issuer.
  audience: "yNmQvjvedarnyxr7LtCvPTmzhwHX0aPJ",
  issuer: `https://systeminit.auth0.com/`,
  algorithms: ["RS256"],
  credentialsRequired: false,
});

const server = new ApolloServer({
  context: (session): typeof session => session,
  modules: [HelloWorld, Users, Workspaces, Integrations],
  introspection: environment.apollo.introspection,
  playground: environment.apollo.playground,
});

const app = express();
app.use(checkJwt);
server.applyMiddleware({ app });

export default app;
