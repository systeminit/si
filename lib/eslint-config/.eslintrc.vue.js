const { execSync } = require('child_process');
const actualProjectDir = execSync('npm prefix').toString().replace(/\n/g, '');
// console.log(actualProjectDir);

module.exports = { 
  env: {
    browser: true,
    es2021: true,
    "vue/setup-compiler-macros": true,
  },
  extends: [
    __dirname + '/.eslintrc.base.js',
    "plugin:vue/vue3-recommended",
    "@vue/eslint-config-typescript/recommended",
    // "airbnb-base",
    // "airbnb-typescript/base",
    "@vue/eslint-config-prettier",
  ],
  parser: "vue-eslint-parser",
  parserOptions: {
    parser: "@typescript-eslint/parser",
    project: [`${actualProjectDir}/tsconfig.json`, `${actualProjectDir}/tsconfig.node.json`],
    // parserOptions: {
    //   ecmaVersion: "latest",
    //   sourceType: "module",
    //   // project: ["./tsconfig.json", "./tsconfig.node.json"],
    //   // TODO: figure our correct settings here
    //   // project: [`${__dirname}/tsconfig.json`],
    // },
  },
  plugins: ["vue"],
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
    "vue/no-multiple-template-root": "error",
  },
}