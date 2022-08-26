module.exports = {
  root: true,
  env: {
    browser: true,
    es2021: true,
    "vue/setup-compiler-macros": true,
  },
  extends: [
    "plugin:vue/vue3-recommended",
    "@vue/eslint-config-typescript/recommended",
    "airbnb-base",
    "airbnb-typescript/base",
    "@vue/eslint-config-prettier",
  ],
  parser: "vue-eslint-parser",
  parserOptions: {
    ecmaVersion: "latest",
    parser: "@typescript-eslint/parser",
    sourceType: "module",
    project: ["./tsconfig.json", "./tsconfig.node.json"],
  },
  plugins: ["vue", "@typescript-eslint", "prettier"],
  settings: {
    "import/resolver": {
      typescript: {},
    },
  },
  rules: {
    camelcase: "off",
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
    "vue/require-default-prop": "off",

    // make import/order understand our alias paths
    "import/order": [
      "warn",
      {
        pathGroups: [
          {
            pattern: "@/**",
            group: "internal",
            position: "after",
          },
        ],
        pathGroupsExcludedImportTypes: ["internal", "external", "builtins"],
        groups: [
          "builtin",
          "external",
          "unknown",
          "internal",
          ["sibling", "parent"],
          "index",
          "object",
          "type",
        ],
      },
    ],

    // some rules from these presets to relax
    "import/prefer-default-export": 0,
    "no-plusplus": 0,
    radix: 0,
    "prefer-destructuring": 0,
    "no-else-return": 0, // sometimes clearer even though unnecessary
    "prefer-arrow-callback": 0,
    "arrow-body-style": 0,
    "@typescript-eslint/lines-between-class-members": 0, // often nice to group related one-liners
    "max-classes-per-file": 0, // can make sense to colocate small classes
    "consistent-return": 0, // often can make sense to return (undefined) early
    "no-useless-return": 0, // sometimes helps clarify you are bailing early
    "no-continue": 0,
    "no-underscore-dangle": 0,
    "no-await-in-loop": 0,
    "no-lonely-if": 0,

    // rules to disable for now, for the sake of fewer changes and eslint disabling...
    // TODO: review these and (maybe) turn back on
    "@typescript-eslint/no-use-before-define": 0,
    "import/no-cycle": 0,
    "no-param-reassign": 0,
    "no-restricted-syntax": 0,
    "@typescript-eslint/naming-convention": 0,
    "@typescript-eslint/no-shadow": 0,
    "guard-for-in": 0,

    // some rules to downgrade to warning while developing
    // useful so things dont crash when code is temporarily commented out
    "@typescript-eslint/no-empty-function": process.env.STRICT_LINT
      ? "error"
      : "warn",
    "prefer-const": process.env.STRICT_LINT ? "error" : "warn",

    "no-console": process.env.STRICT_LINT ? "error" : "warn",
    "no-debugger": process.env.STRICT_LINT ? "error" : "warn",
    "no-alert": process.env.STRICT_LINT ? "error" : "warn",
    "no-empty": process.env.STRICT_LINT ? "error" : "warn",

    // eqeqeq: "error",
    // "object-shorthand": "error",
    // "prefer-template": "error",

    "import/no-duplicates": 0,
  },

  overrides: [
    // overrides for files at the root - which are all for config/build
    {
      files: ["./*"],
      env: { node: true },
      rules: {
        // these files often refer to dev dependencies
        "import/no-extraneous-dependencies": 0,
        // the typescript resolver can't find our mjs files without extension
        "import/extensions": 0,
      },
    },
  ],
};
