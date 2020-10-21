const theme = require("./src/design/themes/tailwind/tailwind-dark");

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
    extend: {
      spacing: theme.spacing,
      colors: theme.colors,
      margin: theme.margin,
    },
  },
  variants: {
    borderColor: ["group-hover"],
    textColor: ["group-hover"],
    backgroundColor: ["odd", "even"],
  },
  plugins: [],
};
