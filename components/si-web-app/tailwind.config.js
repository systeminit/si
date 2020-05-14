const uik = require("./src/design/themes/dark");

module.exports = {
  theme: {
    cursor: {
      resize: "ew-resize",
    },
    fontFamily: {
      sans: ["Roboto", "Sans-serif"],
    },
    extend: {
      colors: uik.colors,
      backgroundColor: "#121212",
    },
  },
  variants: {},
  plugins: [],
};
