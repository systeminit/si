const tailwind = require("tailwindcss");
const autoprefixer = require("autoprefixer");
const tailwindConfig = require("./tailwind.config.js");

module.exports = {
  plugins: [tailwind(tailwindConfig), autoprefixer],
};
