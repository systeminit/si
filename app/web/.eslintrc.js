module.exports = {
  root: true,
  env: {
    node: true,
  },
  globals: { defineProps: "readonly", defineEmits: "readonly" },
  plugins: ["vue", "@typescript-eslint"],
  extends: [
    "plugin:vue/vue3-recommended",
    "eslint:recommended",
    "@vue/typescript/recommended",
    "@vue/prettier",
    "@vue/prettier/@typescript-eslint",
  ],
  rules: {
    camelcase: "off",
    "no-console": "off",
    "no-debugger": "off",
    "no-alert": "error",
    "no-unused-vars": "off", // Causes issues with ts enums
    "@typescript-eslint/no-unused-vars": [
      "warn",
      {
        argsIgnorePattern: "^_",
        varsIgnorePattern: "^_",
      },
    ],
    "vue/script-setup-uses-vars": "error",
    "@typescript-eslint/ban-ts-comment": "off",
  },
  parser: "vue-eslint-parser",
  parserOptions: {
    parser: "@typescript-eslint/parser",
  },
  ignorePatterns: [
    "src/ignore/*",
    "src/organisims/SchematicViewer/references/*",
  ],
};
