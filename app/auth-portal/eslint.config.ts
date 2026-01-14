import { globalIgnores } from "eslint/config";
import {
  defineConfigWithVueTs,
  vueTsConfigs,
} from "@vue/eslint-config-typescript";
import pluginVue from "eslint-plugin-vue";
import pluginVitest from "@vitest/eslint-plugin";
import skipFormatting from "@vue/eslint-config-prettier/skip-formatting";
import importPlugin from "eslint-plugin-import";
import tselint from "typescript-eslint";
import vueParser from "vue-eslint-parser";
import { fileURLToPath } from "url";
import path from "path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// To allow more languages other than `ts` in `.vue` files, uncomment the following lines:
// import { configureVueProject } from '@vue/eslint-config-typescript'
// configureVueProject({ scriptLangs: ['ts', 'tsx'] })
// More info at https://github.com/vuejs/eslint-config-typescript/#advanced-setup

export default defineConfigWithVueTs(
  {
    name: "app/files-to-lint",
    files: ["**/*.{vue,ts,mts,tsx}"],
  },

  globalIgnores([
    "**/dist/**",
    "**/dist-ssr/**",
    "**/coverage/**",
    "eslint.config.ts",
  ]),

  ...pluginVue.configs["flat/essential"],
  vueTsConfigs.recommended,
  importPlugin.flatConfigs["recommended"],
  tselint.configs["recommendedTypeChecked"],
  {
    languageOptions: {
      parser: vueParser,
      parserOptions: {
        parser: "@typescript-eslint/parser",
        // project: [`${__dirname}/tsconfig.json`],
        tsconfigRootDir: __dirname,
        // project: [`./tsconfig.json`, `./tsconfig.node.json`],
        // parserOptions: {
        //   ecmaVersion: "latest",
        //   sourceType: "module",
        //   // project: ["./tsconfig.json", "./tsconfig.node.json"],
        //   // TODO: figure our correct settings here
        //   // project: [`${__dirname}/tsconfig.json`],
        // },
      },
    },
  },

  {
    ...pluginVitest.configs.recommended,
    files: ["src/**/__tests__/*"],
  },

  skipFormatting,

  {
    settings: {
      "import/resolver": {
        node: {
          extensions: [".js", ".jsx", ".ts", ".tsx", ".d.ts"],
        },
        typescript: {
          // Optional: Specify the path to your tsconfig.json if it's not in the root
          project: "./tsconfig.json",
        },
      },
    },
  },

  {
    rules: {
      // dont want this
      "@typescript-eslint/consistent-type-imports": 0,
      "import/named": 0,

      // warning on this because we have some shenanigans where it is a promise but a literal `await` is not present
      "@typescript-eslint/require-await": "warn",

      // "prettier/prettier": "warn",
      "@typescript-eslint/quotes": 0,

      // this is currently breaking, so turning it off
      "@typescript-eslint/unbound-method": 0,

      // some strict rules from TS / airbnb presets to relax -----------
      camelcase: "off",
      // "@typescript-eslint/ban-ts-comment": "off",
      // "import/prefer-default-export": 0,
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
      // curly: ["error", "multi-line"],
      // "brace-style": "error",
      "max-len": [
        "warn", // just a warning since prettier will enforce
        120,
        2,
        {
          // bumped to 120, otherwise same as airbnb's rule but ignoring comments
          ignoreUrls: true,
          ignoreComments: true,
          ignoreRegExpLiterals: true,
          ignoreStrings: true,
          ignoreTemplateLiterals: true,
        },
      ],
      "max-statements-per-line": ["error", { max: 1 }],
      // "@typescript-eslint/no-floating-promises": "error",

      // custom plugin configs ------------------------------------------
      // make import/order understand our alias paths
      // "import/order": [
      //   "warn",
      //   {
      //     pathGroups: [
      //       {
      //         pattern: "@/**",
      //         group: "internal",
      //         position: "after",
      //       },
      //     ],
      //     pathGroupsExcludedImportTypes: ["internal", "external", "builtins"],
      //     groups: [
      //       "builtin",
      //       "external",
      //       "unknown",
      //       "internal",
      //       ["sibling", "parent"],
      //       "index",
      //       "object",
      //       "type",
      //     ],
      //   },
      // ],

      // rules to disable for now, but will likely be turned back on --------
      // TODO: review these rules, infractions case by case, probably turn back on?
      "@typescript-eslint/no-use-before-define": 0,
      // "import/no-cycle": 0,
      "no-param-reassign": 0,
      "no-restricted-syntax": 0,
      "@typescript-eslint/naming-convention": 0,
      "@typescript-eslint/no-shadow": 0,
      "guard-for-in": 0,

      // some rules to downgrade to warning while developing --------------------
      // useful so things dont crash when code is temporarily commented out
      "no-console": "warn",
      "@typescript-eslint/no-empty-function": "warn",
      "no-debugger": "warn",
      "no-alert": "warn",
      "no-empty": "warn",

      // good to warn people
      "@typescript-eslint/no-base-to-string": "warn",
      "@typescript-eslint/restrict-template-expressions": "warn",
      // dont want to error here because we often have async funcs used with other lib calls that dont await, but they do not have to await!
      "@typescript-eslint/no-misused-promises": "warn",

      // turning this off, b/c our instances are actually for meaning & clarity
      "@typescript-eslint/no-redundant-type-constituents": 0,

      // rules that we want to warn, but disable agressive auto-fixing -----------
      "prefer-const": 0,
      "no-unreachable": 0, // handy when you return early or throw an error while debugging
      // unreachable code will be removed by default, so we disable autofix, but leave a warning
      // "no-autofix/no-unreachable": 1,
      // useful while debugging and commenting things out, otherwise gets automatically changed from let to const
      // "no-autofix/prefer-const": "warn",

      "vue/block-order": [
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
      "vue/multi-word-component-names": "off",
      "vue/require-default-prop": "off",
      "vue/padding-line-between-blocks": "error",
      "vue/prefer-true-attribute-shorthand": "error",
      "vue/eqeqeq": "error",
      "vue/no-multiple-template-root": "error",

      "vue/attribute-hyphenation": ["error", "never", { ignore: [] }],
      "vue/v-on-event-hyphenation": "off",

      "@typescript-eslint/ban-ts-comment": 0,
      // skipping for now, but should re-enable
      "@typescript-eslint/no-floating-promises": 0,
    },
  },
);
