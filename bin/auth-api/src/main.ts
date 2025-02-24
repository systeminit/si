/* eslint-disable no-console */

// import first to set up env
import "./init-env";

import Koa from "koa";
import bodyParser from 'koa-bodyparser';
import chalk from 'chalk';
import cors from '@koa/cors';

import { PrismaClient } from "@prisma/client";
import { router, automationApiRouter, routesLoaded } from "./routes";
import { ApiError, errorHandlingMiddleware } from "./lib/api-error";
import { httpRequestLoggingMiddleware } from "./lib/request-logger";
import { loadAuthMiddleware, requireWebTokenMiddleware } from "./services/auth.service";
import { detectClientIp } from "./lib/client-ip";
import { CustomAppContext, CustomAppState } from "./custom-state";

import './lib/posthog';

export const prisma = new PrismaClient();

export const app = new Koa<CustomAppState, CustomAppContext>();

app.proxy = true;

const allowedOrigins = new Set([
  "https://tools.systeminit.com",
  "https://app.systeminit.com",
  "https://auth.systeminit.com",
  "https://auth-api.systeminit.com",
  "https://module-index.systeminit.com",
]);

// include this one early since it can fire off and be done when handling OPTIONS requests
// only allow permitted origins within cors
app.use(cors({
  origin: (ctx) => {
    const requestOrigin = ctx.request.header.origin || "";
    if (process.env.NODE_ENV !== 'test') {
      return allowedOrigins.has(requestOrigin) ? requestOrigin : "null";
    } else {
      return "null"
    }
  },
  credentials: true, // Only send credentials for allowed origins
}));

app.use(detectClientIp);
app.use(httpRequestLoggingMiddleware);
app.use(errorHandlingMiddleware);
app.use(bodyParser());
app.use(loadAuthMiddleware);
// routes - must be last after all middlewares
app.use(automationApiRouter.routes());
app.use(router.use(requireWebTokenMiddleware).routes());

// catch-all middelware after routes handles no route match (404)
app.use((_ctx, _next) => {
  throw new ApiError("NotFound", "NoMatchingURL", "No matching URL found");
});

if (process.env.NODE_ENV !== 'test') {
  // not strictly necessary, but this way we fail right away if we can't connect to db
  try {
    await prisma.$connect();
    await routesLoaded;
    app.listen(process.env.PORT);
    console.log(chalk.green.bold(`Auth API listening on port ${process.env.PORT}`));
    // await prisma.$disconnect();
  } catch (err) {
    console.log('ERROR!', err);
    await prisma.$disconnect();
  }
}
