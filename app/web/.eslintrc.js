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
  plugins: ["vue", "@typescript-eslint", "prettier", "no-autofix"],
  settings: {
    "import/resolver": {
      typescript: {},
    },
  },
  rules: {
    // some customizations of vue rules ------------------
    // standard order of sections in vue SFCs
    "vue/component-tags-order": [
      "error",
      {
        order: [
          "template",
          "script[setup]",
          "script:not([setup])", // necessary for default exports to not get hoisted below imports in setup block
          "style:not([scoped])",
          "style[scoped]",
        ],
      },
    ],
    "vue/no-undef-components": [
      "error",
      {
        ignorePatterns: [
          "v-.*", // vue-konva requires global registration :( will hopefully fix soon!
          "router-(view|link)", // vue router is fairly standard to use via global registration
        ],
      },
    ],
    "vue/script-setup-uses-vars": "error",
    "vue/multi-word-component-names": "off",
    "vue/require-default-prop": "off",
    "vue/padding-line-between-blocks": "error",
    "vue/prefer-true-attribute-shorthand": "error",
    "vue/eqeqeq": "error",

    // some strict rules from TS / airbnb presets to relax -----------
    camelcase: "off",
    "@typescript-eslint/ban-ts-comment": "off",
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
    "@typescript-eslint/no-unused-vars": [
      "warn",
      {
        argsIgnorePattern: "^_",
        varsIgnorePattern: "^_|^(props)$",
      },
    ],

    // custom plugin configs ------------------------------------------
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

    // rules to disable for now, but will likely be turned back on --------
    // TODO: review these rules, infractions case by case, probably turn back on?
    "@typescript-eslint/no-use-before-define": 0,
    "import/no-cycle": 0,
    "no-param-reassign": 0,
    "no-restricted-syntax": 0,
    "@typescript-eslint/naming-convention": 0,
    "@typescript-eslint/no-shadow": 0,
    "guard-for-in": 0,
    "no-console": 0,

    // some rules to downgrade to warning while developing --------------------
    // useful so things dont crash when code is temporarily commented out
    "@typescript-eslint/no-empty-function": process.env.STRICT_LINT
      ? "error"
      : "warn",
    "no-debugger": process.env.STRICT_LINT ? "error" : "warn",
    "no-alert": process.env.STRICT_LINT ? "error" : "warn",
    "no-empty": process.env.STRICT_LINT ? "error" : "warn",
    // "no-console": process.env.STRICT_LINT ? "error" : "warn",

    // rules that we want to warn, but disable agressive auto-fixing
    "prefer-const": 0,
    "no-unreachable": 0, // handy when you return early or throw an error while debugging
    // unreachable code will be removed by default, so we disable autofix, but leave a warning
    "no-autofix/no-unreachable": 1,
    // useful while debugging and commenting things out, otherwise gets automatically changed from let to const
    "no-autofix/prefer-const": process.env.STRICT_LINT ? "error" : "warn",
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
