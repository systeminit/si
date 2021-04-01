import Koa, { Context } from "koa";
import Router from "koa-router";
import websocket from "koa-easy-ws";
import logger from "koa-logger";
import json from "koa-json";
import koaBody from "koa-body";
import controller from "./controllers";
import { BehaviorSubject } from "rxjs";
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
