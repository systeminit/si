import colors from "tailwindcss/colors";
import formsPlugin from "@tailwindcss/forms";
import * as themeValues from "./src/assets/style/tailwind_customization/theme_values.mjs";
import typographyPlugin from "./src/assets/style/tailwind_customization/typography_plugin.mjs";

export default {
  darkMode: "class",
  content: ["./src/**/*.html", "./src/**/*.vue"],
  theme: {
    cursor: {
      resize: "ew-resize",
      pointer: "pointer",
      move: "move",
    },
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
