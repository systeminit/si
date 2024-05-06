/* eslint-disable import/no-extraneous-dependencies */
const formsPlugin = require("@tailwindcss/forms");
const colors = require("tailwindcss/colors");
const capsizePlugin = require("tailwindcss-capsize");
const lineClampPlugin = require("@tailwindcss/line-clamp");
const headlessUiPlugin = require("@headlessui/tailwindcss");

const themeValues = require("./tailwind_customization/tailwind_theme_values.cjs");
const typographyPlugin = require("./tailwind_customization/typography_plugin.cjs");
const childrenVariantPlugin = require("./tailwind_customization/children_variant_plugin.cjs");

/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: "class",
  content: ["src/**/*.vue", "node_modules/@si/vue-lib/**/*.vue"],
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
        md: "1rem",
        "2xs": "0.6rem",
        "3xs": "0.5rem",
      },
      transitionDelay: {
        0: "0ms",
        2000: "2000ms",
      },
      transitionDuration: {
        0: "0ms",
      },
      boxShadow: {
        "3xl": "0px 4px 15px 0px rgba(0, 0, 0, 0.55)",
      },
      animation: {
        "spin-cc": "spin-cc 1s linear infinite",
      },
      keyframes: {
        "spin-cc": {
          from: {
            transform: "rotate(360deg)",
          },
        },
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
  safelist: [
    "text-2xs", // TODO - this can be removed when we use 'text-2xs' somewhere that isn't generated
  ],
};
