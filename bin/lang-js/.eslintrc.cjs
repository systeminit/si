module.exports = {
  extends: ["@si/eslint-config/base"],
  overrides: [
    {
      files: ["tests/functions/*.ts"],
      rules: {
        "@typescript-eslint/no-unused-vars": "off"
      }
    }
  ],
};
