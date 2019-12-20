module.exports = {
  env: {
    node: true,
  },
  extends: ["plugin:vue/essential", "@vue/prettier", "@vue/typescript"],
  plugins: ["graphql", "vuetify"],
  rules: {
    "no-console": process.env.NODE_ENV === "production" ? "error" : "off",
    "no-debugger": process.env.NODE_ENV === "production" ? "error" : "off",
    "vuetify/no-deprecated-classes": "error",
    "vuetify/grid-unknown-attributes": "error",
    "vuetify/no-legacy-grid": "error",
  },
  parserOptions: {
    parser: "@typescript-eslint/parser",
  },
};
