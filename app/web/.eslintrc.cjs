module.exports = {
  extends: ["@si/eslint-config/vue"],
  rules: {
    "@typescript-eslint/ban-ts-comment": 0,
    // skipping for now, but should re-enable
    "@typescript-eslint/no-floating-promises": 0,
  },
  overrides: [
    // overrides for files at the root - which are all for config/build
    {
      files: ["./*", "./build-src/*"],
      env: { node: true },
      rules: {
        // these files often refer to dev dependencies
        "import/no-extraneous-dependencies": 0,
        "import/extensions": 0,
      },
    },
    {
      files: ["./src/newhotness/testing/*"],
      env: { node: true },
      rules: {
        // these files often refer to dev dependencies
        "import/no-extraneous-dependencies": 0,
      },
    },
  ],
};
