import Koa, { Context } from "koa";
import Router from "koa-router";
import websocket from "koa-easy-ws";
import logger from "koa-logger";
import json from "koa-json";
import koaBody from "koa-body";
import controller from "./controllers";
import Debug from "debug";
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
  if (ctx.ws) {
    const ws = await ctx.ws();

    ws.on("message", function (msg: string) {
      controller.checkQualifications(ws, msg);
    });
    ws.on("close", (code: number, reason: string) => {
      debug("socket closed", { code, reason });
    });
  }
});
router.get("/runCommand", async (ctx: Context) => {
  if (ctx.ws) {
    const ws = await ctx.ws();

    ws.on("message", function (msg: string) {
      controller.runCommand(ws, msg);
    });
    ws.on("close", (code: number, reason: string) => {
      debug("socket closed", { code, reason });
    });
  }
});
router.get("/syncResource", async (ctx: Context) => {
  if (ctx.ws) {
    const ws = await ctx.ws();

    ws.on("message", function (msg: string) {
      controller.syncResource(ws, msg);
    });
    ws.on("close", (code: number, reason: string) => {
      debug("socket closed", { code, reason });
    });
  }
});
