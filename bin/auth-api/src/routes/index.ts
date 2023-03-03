import * as url from "url";
import glob from "glob";
import Router from "@koa/router";

const __filename = url.fileURLToPath(import.meta.url);
const __dirname = url.fileURLToPath(new URL(".", import.meta.url));

// we initialize and export the router immediately
// but we'll add routes to it here and in each routes file
export const router = new Router();

router.get("/", async (ctx) => {
  ctx.body = { systemStatus: "ok" };
});

router.get("/boom", async (ctx) => {
  throw new Error("Moooo!");
});

// automatically load all *.routes.ts files in this directory
const routeFilePaths = glob.sync(`${__dirname}/**/*.routes.ts`);
routeFilePaths.forEach((routeFilePath) => {
  const load = import(routeFilePath.replace(__dirname, "./"));
  // technically loading async, but should not be a problem
  load.then((file) => file.initRoutes(router));
});
