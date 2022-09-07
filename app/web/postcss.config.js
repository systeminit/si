const tailwind = require("tailwindcss");
const autoprefixer = require("autoprefixer");
const nestingPlugin = require("tailwindcss/nesting");
const loopPlugin = require("postcss-each");
const postcssCommentParser = require("postcss-comment");

const tailwindConfig = require("./tailwind.config.js");

module.exports = {
  parser: postcssCommentParser,
  plugins: [nestingPlugin, loopPlugin, tailwind(tailwindConfig), autoprefixer],
};
