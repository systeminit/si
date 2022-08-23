import * as theme from "./src/design/themes/tailwind.mjs";
import colors from "tailwindcss/colors";
import formsPlugin from "@tailwindcss/forms";
import typographyPlugin from "./src/design/themes/tailwind/plugin/si_typography";

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
      spacing: theme.spacing,
      colors: theme.colors,
      margin: theme.margin,
      maxHeight: theme.maxHeight,
      zIndex: theme.zIndex,
      width: theme.width,
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
