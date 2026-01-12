import _ from "lodash";
import chalk from "chalk";
import { v4 as uuidv4 } from "uuid";
import Koa from 'koa';

// import { RouteCtx } from "./router";
import { log } from "./logger";

// Koa request middleware that
// - sets up a generic logging function that attaches extra info about the request
// - calls ctx.log to write a final log of the request along with what happened (url, status code, timer)
// TODO: probably rewrite this using ALS
export const httpRequestLoggingMiddleware: Koa.Middleware = async (ctx, next) => {
  const { req, res } = ctx;

  // skip logs for favicon, which browsers hit when hitting endpoints directly
  if (req.url === "/favicon.ico") return next();

  ctx.state.requestStart = +new Date();
  ctx.state.requestId = uuidv4();

  const requestInfo: Record<string, any> = {
    url: req.url,
    method: req.method,
    requestId: ctx.state.requestId,
    userAgent: req.headers["user-agent"],
    remoteIp: ctx.state.clientIp,
    // apiVersion: ctx.state.version,
    ...(ctx.request.originalUrl && { originalUrl: ctx.request.originalUrl }),
  };

  // set up general "ctx.logContext" object to add any contextual info about the request
  // which will be attached to all log calls
  // this is designed to be filled in later by the actual request handler
  ctx.logContext = {};

  // define a log method which attaches a extra info about the request
  ctx.log = (message: string, meta: any) => {
    log(message, {
      type: "http",
      ...requestInfo,
      // ...(ctx.state.authUser && {
      //   user: {
      //     type: ctx.state.authUser.ModelName.toLowerCase(), // 'user' or 'admin'
      //     ..._.pick(ctx.state.authUser, "id", "email"),
      //   },
      // }),
      ...(!_.isEmpty(ctx.logContext) && { context: ctx.logContext }),
      ...meta,
    });
  };

  await next(); // pass control back to Koa and process the actual request

  const requestTimeSpent = +new Date() - ctx.state.requestStart;

  // skip logs for ping/health checks
  const isHealthCheck = requestInfo.url === "/" && (
    requestInfo.userAgent?.startsWith("ELB-HealthChecker/")
    || requestInfo.userAgent?.startsWith("Render/")
  );
  if (isHealthCheck) {
    return;
  }

  // write final log of request
  ctx.log(
    `${requestInfo.method} ${requestInfo.url} ${chalk[
      res.statusCode < 400 ? "green" : "red"
    ](res.statusCode)} ${requestTimeSpent}ms`,
    {
      timer: requestTimeSpent,
      statusCode: res.statusCode,

      // this passes through any captured error to the log fn
      // which includes info in the logdna log but also passes through to sentry
      ...(ctx.state.capturedError && { error: ctx.state.capturedError }),
    },
  );
};
