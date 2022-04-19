module.exports = {
  env: {
    browser: true,
    es2021: true,
  },
  extends: [
    "plugin:nuxt/recommended",
    "plugin:vue/vue3-recommended",
    "plugin:prettier/recommended",
  ],
  parserOptions: {
    parser: "@typescript-eslint/parser",
  },
  plugins: ["vue", "@typescript-eslint"],
  rules: {
    "vue/multi-word-component-names": "off",
    "no-unused-vars": "warn",
    "space-in-parens": "off",
    "computed-property-spacing": "off",
    "max-len": "off",
  },
};
