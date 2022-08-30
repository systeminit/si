const formsPlugin = require("@tailwindcss/forms");
const colors = require("tailwindcss/colors");
const themeValues = require("./src/assets/style/tailwind_customization/theme_values.js");
const typographyPlugin = require("./src/assets/style/tailwind_customization/typography_plugin.js");

module.exports = {
  darkMode: "class",
  content: ["./src/**/*.html", "./src/**/*.vue"],
  theme: {
    fontFamily: {
      sans: ["Inter", "Sans-serif"],
      commodore: ["commodore", "Sans-serif"],
    },
    colors: {
      transparent: "transparent",
      current: "currentColor",
      black: colors.black,
      white: colors.white,
      gray: colors.neutral,
      blue: colors.blue,
      blueGray: colors.slate,
      indigo: colors.indigo,
      red: colors.rose,
      yellow: colors.amber,
      green: colors.green,
    },
    extend: {
      spacing: themeValues.spacing,
      colors: themeValues.colors,
      margin: themeValues.margin,
      maxHeight: themeValues.maxHeight,
      zIndex: themeValues.zIndex,
      width: themeValues.width,
    },
  },
  variants: {
    borderColor: ["group-hover"],
    textColor: ["group-hover"],
    backgroundColor: ["odd", "even"],
    extend: {
      opacity: ["disabled"],
    },
  },
  plugins: [formsPlugin, typographyPlugin],
};
