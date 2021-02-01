var fs = require("fs");
var path = require("path");

module.exports = {
  root: true,
  env: {
    node: true,
  },
  extends: ["plugin:vue/essential", "@vue/prettier", "@vue/typescript"],
  rules: {
    "no-console": "off",
    "no-debugger": "off",
    "no-alert": "error",
  },
  parserOptions: {
    parser: "@typescript-eslint/parser",
  },
};
