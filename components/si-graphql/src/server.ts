import express from "express";
import { ApolloServer } from "apollo-server-express";
import jwt from "express-jwt";
import jwksRsa from "jwks-rsa";
import cors from "cors";

import { AppModule } from "@/app.modules";
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
  modules: [AppModule],
  introspection: environment.apollo.introspection,
  playground: environment.apollo.playground,
});

const app = express();
app.use(cors());
app.options("*", cors());
app.use(checkJwt);
server.applyMiddleware({ app });

app.get("/", function(_req, res, _next): void {
  res.send(`
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>System Initiative GraphQL API</title>
</head>
<body>
<h1>The System Initiative GraphQL API</h1>
</body>
</html>
`);
});

export default app;
