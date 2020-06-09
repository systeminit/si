var fs = require("fs");
var path = require("path");

module.exports = {
  root: true,
  env: {
    node: true,
  },
  extends: ["plugin:vue/essential", "@vue/prettier", "@vue/typescript"],
  rules: {
    "graphql/template-strings": [
      "error",
      {
        env: "literal",
        schemaString: fs.readFileSync(
          path.resolve(__dirname, "../si-graphql-api/fullstack-schema.graphql"),
          { encoding: "utf8" },
        ),
      },
    ],
    "no-console": process.env.NODE_ENV === "production" ? "error" : "off",
    "no-debugger": process.env.NODE_ENV === "production" ? "error" : "off",
  },
  parserOptions: {
    parser: "@typescript-eslint/parser",
  },
  plugins: ["graphql"],
};