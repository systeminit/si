module.exports = {
  root: true,
  env: {
    node: true,
  },
  globals: {
    defineProps: "readonly",
    defineEmits: "readonly",
    defineExpose: "readonly",
    withDefaults: "readonly",
  },
  plugins: ["vue", "@typescript-eslint"],
  extends: [
    "plugin:vue/vue3-recommended",
    "eslint:recommended",
    "@vue/typescript/recommended",
    "@vue/prettier",
  ],
  rules: {
    camelcase: "off",
    "no-console": "off",
    "no-debugger": "off",
    "no-alert": "error",
    "@typescript-eslint/no-unused-vars": [
      "warn",
      {
        argsIgnorePattern: "^_",
        varsIgnorePattern: "^_",
      },
    ],
    "vue/script-setup-uses-vars": "error",
    "@typescript-eslint/ban-ts-comment": "off",
    "vue/multi-word-component-names": "off",
    "vue/v-on-event-hyphenation": "off",
    "vue/require-default-prop": "off",
    // some rules to downgrade to warning while developing
    // useful so things dont crash when code is temporarily commented out
    "@typescript-eslint/no-empty-function": process.env.STRICT_LINT
      ? "error"
      : "warn",
  },
  parser: "vue-eslint-parser",
  parserOptions: {
    parser: "@typescript-eslint/parser",
  },
  ignorePatterns: [
    "src/ignore/*",
    "src/organisms/SiCanvas/references/*",
  ],
};
