module.exports = {
  extends: ["@si/eslint-config/base"],
  overrides: [
    {
      files: ["tests/functions/*.ts"],
      rules: {
        "@typescript-eslint/no-unused-vars": "off",
      },
    },
    {
      files: ["**/*.ts"],
      rules: {
        "no-console": "off",
      },
    },
  ],
  parserOptions: {
    parser: "@typescript-eslint/parser",
    project: "./tsconfig.json",
    tsconfigRootDir: __dirname,
  },
};
