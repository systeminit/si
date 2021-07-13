import Koa, { Context } from "koa";
import Router from "koa-router";
import websocket from "koa-easy-ws";
import logger from "koa-logger";
import json from "koa-json";
import koaBody from "koa-body";
import controller from "./controllers";
import Debug from "debug";
import "./telemetry";
import api from "@opentelemetry/api";
const debug = Debug("veritech");

export const app = new Koa();
const router = new Router();
app.use(websocket());
app.use(koaBody());
app.use(json());
app.use(logger());
app.use(router.routes());
app.use(router.allowedMethods());

router.post("/inferProperties", controller.inferProperties);
router.post("/loadWorkflows", controller.loadWorkflows);
router.get("/checkQualifications", async (ctx: Context) => {
  const span = api.trace.getSpan(api.context.active());
  span.updateName("veritech.checkqualifications");

  if (ctx.ws) {
    const ws = await ctx.ws();

    ws.on("message", function (msg: string) {
      controller.checkQualifications(ws, msg, span);
    });
    ws.on("close", (code: number, reason: string) => {
      debug("socket closed", { code, reason });
      span.end();
    });
  }
});
router.get("/runCommand", async (ctx: Context) => {
  const span = api.trace.getSpan(api.context.active());
  span.updateName("veritech.runcommand");

  if (ctx.ws) {
    const ws = await ctx.ws();

    ws.on("message", function (msg: string) {
      controller.runCommand(ws, msg, span);
    });
    ws.on("close", (code: number, reason: string) => {
      debug("socket closed", { code, reason });
      span.end();
    });
  }
});
router.get("/syncResource", async (ctx: Context) => {
  const span = api.trace.getSpan(api.context.active());
  span.updateName("veritech.syncresource");

  if (ctx.ws) {
    const ws = await ctx.ws();

    ws.on("message", function (msg: string) {
      controller.syncResource(ws, msg, span);
    });
    ws.on("close", (code: number, reason: string) => {
      debug("socket closed", { code, reason });
      span.end();
    });
  }
});
router.get("/discover", async (ctx: Context) => {
  const span = api.trace.getSpan(api.context.active());
  span.updateName("veritech.discover");

  if (ctx.ws) {
    const ws = await ctx.ws();

    ws.on("message", function (msg: string) {
      controller.discover(ws, msg, span);
    });
    ws.on("close", (code: number, reason: string) => {
      debug("socket closed", { code, reason });
      span.end();
    });
  }
});
