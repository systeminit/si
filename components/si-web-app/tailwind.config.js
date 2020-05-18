const theme = require("./src/design/themes/tailwind/tailwind-dark");

module.exports = {
  theme: {
    cursor: {
      resize: "ew-resize",
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
