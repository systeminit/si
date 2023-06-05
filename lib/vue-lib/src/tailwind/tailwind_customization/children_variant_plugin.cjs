/* eslint-disable import/no-extraneous-dependencies */
const plugin = require("tailwindcss/plugin");

const childrenVariantPlugin = plugin(({ addVariant }) => {
  addVariant("children", "& > *");
});

module.exports = childrenVariantPlugin;
