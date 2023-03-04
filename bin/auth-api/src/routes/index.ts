import glob from "glob";
import Router from "@koa/router";
import { getThisDirname } from "../lib/this-file-path";

const __dirname = getThisDirname(import.meta.url);

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
  // technically loading is async, but is not actually a problem
  load.then((file) => file.initRoutes(router)); // eslint-disable-line @typescript-eslint/no-floating-promises
});
