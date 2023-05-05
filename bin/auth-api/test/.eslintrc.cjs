module.exports = {
  rules: {
    'no-console': 0,
    'import/no-extraneous-dependencies': 0,
    // doesn't like expect(asdf).to.be.true styles statements
    '@typescript-eslint/no-unused-expressions': 0,

    // we dont need to await the t.test fns
    '@typescript-eslint/no-floating-promises': 0,
    'padded-blocks': 0,
  },
};

