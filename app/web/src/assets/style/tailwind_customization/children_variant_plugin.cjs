const plugin = require("tailwindcss/plugin");

const childrenVariantPlugin = plugin(({ addVariant }) => {
  addVariant("children", "& > *");
});

module.exports = childrenVariantPlugin;
