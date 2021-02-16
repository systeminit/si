const theme = require("./src/design/themes/tailwind/tailwind-dark");
const colors = require("tailwindcss/colors");

module.exports = {
  purge: ["./src/**/*.html", "./src/**/*.vue"],
  theme: {
    cursor: {
      resize: "ew-resize",
      pointer: "pointer",
      move: "move",
    },
    fontFamily: {
      sans: ["Roboto", "Sans-serif"],
    },
    colors: {
      transparent: "transparent",
      current: "currentColor",
      black: colors.black,
      white: colors.white,
      gray: colors.trueGray,
      blue: colors.blue,
      blueGray: colors.blueGray,
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
    },
  },
  variants: {
    borderColor: ["group-hover"],
    textColor: ["group-hover"],
    backgroundColor: ["odd", "even"],
  },
  plugins: [],
};
