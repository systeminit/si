const formsPlugin = require("@tailwindcss/forms");
const colors = require("tailwindcss/colors");
const capsizePlugin = require("tailwindcss-capsize");
const lineClampPlugin = require("@tailwindcss/line-clamp");
const headlessUiPlugin = require("@headlessui/tailwindcss");
const themeValues = require("./src/assets/style/tailwind_customization/tailwind_theme_values.js");
const typographyPlugin = require("./src/assets/style/tailwind_customization/typography_plugin.js");
const childrenVariantPlugin = require("./src/assets/style/tailwind_customization/children_variant_plugin");

module.exports = {
  darkMode: "class",
  content: ["./src/**/*.vue"],
  theme: {
    fontFamily: {
      sans: ["Inter", "sans-serif"],
    },
    fontMetrics: {
      sans: {
        capHeight: 2048,
        ascent: 2728,
        descent: -680,
        lineGap: 0,
        unitsPerEm: 2816,
        xHeight: 1536,
      },
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
      // TODO: change from extends to override once we remove references to those sizes
      screens: themeValues.screens,

      fontSize: {
        "2xs": "0.5rem",
      },
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
  plugins: [
    capsizePlugin,
    formsPlugin,
    typographyPlugin,
    lineClampPlugin,
    childrenVariantPlugin,
    headlessUiPlugin,
  ],
};
