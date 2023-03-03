import Router from "@koa/router";

export const router = new Router();

router.get("/foo", async (ctx) => {
  ctx.body = { bar: 4 };
});

router.get("/", async (ctx) => {
  ctx.body = { ok: true };
});
