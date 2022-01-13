module.exports = {
  root: true,
  env: {
    node: true,
  },
  globals: { defineProps: "readonly", defineEmits: "readonly" },
  extends: [
    "eslint:recommended",
    "plugin:vue/vue3-recommended",
    "@vue/typescript",
    "@vue/prettier",
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
  },
  parserOptions: {
    parser: "@typescript-eslint/parser",
  },
  ignorePatterns: [
    "src/ignore/*",
    "src/organisims/SchematicViewer/references/*",
  ],
};
