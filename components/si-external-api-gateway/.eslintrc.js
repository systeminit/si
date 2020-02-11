module.exports = {
  parser:  '@typescript-eslint/parser',  // Specifies the ESLint parser
  extends:  [
    'plugin:@typescript-eslint/recommended',  // Uses the recommended rules from the @typescript-eslint/eslint-plugin
    'prettier/@typescript-eslint',
    'plugin:prettier/recommended'
  ],
  parserOptions:  {
    ecmaVersion:  2018,  // Allows for the parsing of modern ECMAScript features
    sourceType:  'module',  // Allows for the use of imports
  },
  rules:  {
    "no-unused-vars": [2, {"args": "after-used", "argsIgnorePattern": "^_"}],
    "@typescript-eslint/ban-ts-ignore": "off",
    "@typescript-eslint/no-unused-vars": [2, {"args": "after-used", "argsIgnorePattern": "^_"}],
    // Place to specify ESLint rules. Can be used to overwrite rules specified from the extended configs
    // e.g. "@typescript-eslint/explicit-function-return-type": "off",
  },
  overrides: [
    {
      files: [ "si.*.js" ],
      env: { node: true, es6: true },
      parser: "espree",
      parserOptions: {
        ecmaVersion: 2018,
        sourceType: 'module',
      },
      extends: [ 
        "eslint:recommended",
        "plugin:prettier/recommended",
      ],
      rules: {
        "no-undef": "off",
        "no-var": "off",
        "no-redeclare": "off",
        "prefer-const": "off",
        "no-unused-vars": "off",
        "@typescript-eslint/explicit-function-return-type": "off",
        "@typescript-eslint/no-unused-vars": "off",
        "@typescript-eslint/camelcase": "off",
        "@typescript-eslint/no-var-requires": "off",
      }
    }
  ],
}
