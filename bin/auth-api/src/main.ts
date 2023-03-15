/* eslint-disable no-console */

// import first to set up env
import "./init-env";

import Koa from "koa";
import bodyParser from 'koa-bodyparser';
import chalk from 'chalk';
import cors from '@koa/cors';

import { PrismaClient } from "@prisma/client";
import { router } from "./routes";
import { ApiError, errorHandlingMiddleware } from "./lib/api-error";
import { httpRequestLoggingMiddleware } from "./lib/request-logger";
import { loadAuthMiddleware } from "./services/auth.service";
import { detectClientIp } from "./lib/client-ip";
import { CustomAppContext, CustomAppState } from "./custom-state";

const prisma = new PrismaClient();

const app = new Koa<CustomAppState, CustomAppContext>();

// include this one early since it can fire off and be done when handling OPTIONS requests
app.use(cors({ credentials: true }));
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

async function boot() {
  // not strictly necessary, but this way we fail right away if we can't connect to db
  await prisma.$connect();
  app.listen(process.env.PORT);
  console.log(chalk.green.bold(`Auth API listening on port ${process.env.PORT}`));
}

boot()
  .then(async () => {
    await prisma.$disconnect();
  })
  .catch(async (err) => {
    console.log('ERROR!', err);
    await prisma.$disconnect();
  });
