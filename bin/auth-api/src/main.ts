// import first to set up env
import "./init-env";

import Koa from "koa";
import { koaBody } from "koa-body";
import chalk from "chalk";

import { router } from "./routes";
import { ApiError, errorHandlingMiddleware } from "./lib/api-error";
import { httpRequestLoggingMiddleware } from "./lib/request-logger";

const app = new Koa();

app.use(httpRequestLoggingMiddleware);
app.use(errorHandlingMiddleware);
app.use(koaBody());

// routes - must be last after all middlewares
app.use(router.routes());

// catch-all middelware after routes handles no route match (404)
app.use((_ctx, _next) => {
  throw new ApiError("NotFound", "URL not found");
});

app.listen(9001);
console.log(chalk.green.bold("Auth API listening on port 9001"));
