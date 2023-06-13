const { execSync } = require('child_process');
const actualProjectDir = execSync('npm prefix').toString().replace(/\n/g, '');

module.exports = {
  env: {
    node: true,
  },
  parser: "@typescript-eslint/parser",
  parserOptions: {
    project: [`${actualProjectDir}/tsconfig.json`],
    tsconfigRootDir: __dirname, 
  },
  plugins: [
    "@typescript-eslint", 
    // "prettier",
    "no-autofix",  
  ],
  extends: [
    "airbnb-base",
    "airbnb-typescript/base",
    // "plugin:prettier/recommended",
  ],
  settings: {
    "import/resolver": {
      typescript: {},
      // fixes some issues with packages that have multiple exports via package.json exports field
      exports: {},
    },
  },
  rules: {
    // "prettier/prettier": process.env.STRICT_LINT ? "error" : "warn",
    "@typescript-eslint/quotes": 0,

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
        argsIgnorePattern: "^_|^(response)$",
        varsIgnorePattern: "^_|^(props|emit)$",
      },
    ],
    "@typescript-eslint/return-await": 0,

    // other -----------------------------------------------------
    "no-undef": 0, // handled by typescript, which is better aware of global types
    curly: ["error", "multi-line"],
    "brace-style": "error",
    'max-len': ['error', 120, 2, { // bumped to 120, otherwise same as airbnb's rule but ignoring comments
      ignoreUrls: true,
      ignoreComments: true,
      ignoreRegExpLiterals: true,
      ignoreStrings: true,
      ignoreTemplateLiterals: true,
    }],
    "max-statements-per-line": ["error", { "max": 1 }],
    "@typescript-eslint/no-floating-promises": "error",


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

    // some rules to downgrade to warning while developing --------------------
    // useful so things dont crash when code is temporarily commented out
    "no-console": process.env.STRICT_LINT ? "error" : "warn",
    "@typescript-eslint/no-empty-function": process.env.STRICT_LINT
      ? "error"
      : "warn",
    "no-debugger": process.env.STRICT_LINT ? "error" : "warn",
    "no-alert": process.env.STRICT_LINT ? "error" : "warn",
    "no-empty": process.env.STRICT_LINT ? "error" : "warn",

    
    // rules that we want to warn, but disable agressive auto-fixing -----------
    "prefer-const": 0,
    "no-unreachable": 0, // handy when you return early or throw an error while debugging
    // unreachable code will be removed by default, so we disable autofix, but leave a warning
    "no-autofix/no-unreachable": 1,
    // useful while debugging and commenting things out, otherwise gets automatically changed from let to const
    "no-autofix/prefer-const": process.env.STRICT_LINT ? "error" : "warn",

  },
}
