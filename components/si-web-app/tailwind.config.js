const theme = require("./src/design/themes/tailwind/tailwind-dark");

module.exports = {
  theme: {
    cursor: {
      resize: "ew-resize",
      pointer: "pointer",
      move: "move"
    },
    fontFamily: {
      sans: ["Roboto", "Sans-serif"],
    },
    extend: {
      spacing: theme.spacing,
      colors: theme.colors,
    },
  },
  variants: {
    borderColor: ["group-hover"],
    textColor: ["group-hover"],
  },
  plugins: [],
};
