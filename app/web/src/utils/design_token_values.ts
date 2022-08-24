import _ from "lodash";
// @ts-ignore
import resolveConfig from "tailwindcss/resolveConfig";
// @ts-ignore
import tailwindConfig from "../../tailwind.config.mjs";

const fullConfig = resolveConfig(tailwindConfig);

export const colors = _.pick(fullConfig.theme.colors, [
  "neutral",
  "action",
  "success",
  "warning",
  "destructive",
]);
