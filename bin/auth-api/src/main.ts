import Koa from "koa";
import { router } from "./routes";

const app = new Koa();

app.use(async (ctx, next) => {
  console.log("middleware!");
  return next();
});

app.use(router.routes());

// catch-all middelware after routes handles no route match (404)
app.use((ctx, next) => {
  ctx.status = 404;
  ctx.body = {
    error: {
      code: "NotFound",
      message: "URL not found",
    },
  };
});

app.listen(9001);
console.log("Auth API listening on port 9001");
