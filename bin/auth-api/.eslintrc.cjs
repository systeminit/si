module.exports = {
  extends: ["@si/eslint-config/base"],
  rules: {
    // these dont actually exist in our weird node/esm setup
    "no-restricted-globals": ["error", "__filename", "__dirname"],

    "@typescript-eslint/no-unused-vars": [
      "warn",
      {
        argsIgnorePattern: "^_|^(ctx|next)$",
        // varsIgnorePattern: "^_|^(props|emit)$",
      },
    ],
  },
};
