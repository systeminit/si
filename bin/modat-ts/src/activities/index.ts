import { Context } from "../context";
import { greet } from "./greet";

export const createActivities = (ctx: Context) => ({
  greet: greet.bind(null, ctx),
});
