// import first to set up env
import "./init-env";

import Koa from "koa";
import bodyParser from 'koa-bodyparser';
import chalk from 'chalk';
import cors from '@koa/cors';
import requestIp from 'request-ip';

import { router } from "./routes";
import { ApiError, errorHandlingMiddleware } from "./lib/api-error";
import { httpRequestLoggingMiddleware } from "./lib/request-logger";
import { loadAuthMiddleware } from "./services/auth.service";
import { detectClientIp } from "./lib/client-ip";
import { User } from "./services/users.service";
import { Workspace } from "./services/workspaces.service";
import { CustomAppContext, CustomAppState } from "./custom-state";

const app = new Koa<CustomAppState, CustomAppContext>();

// include this one early since it can fire off and be done when handling OPTIONS requests
app.use(cors({
  credentials: true,
}));

app.use(detectClientIp);
app.use(httpRequestLoggingMiddleware);
app.use(errorHandlingMiddleware);

app.use(bodyParser());
app.use(loadAuthMiddleware);

// routes - must be last after all middlewares
app.use(router.routes());

// catch-all middelware after routes handles no route match (404)
app.use((_ctx, _next) => {
  throw new ApiError("NotFound", "URL not found");
});

app.listen(9001);
console.log(chalk.green.bold("Auth API listening on port 9001"));
