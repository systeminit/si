// TODO: set up nice way to keep these values synced with less vars
const colors = require("./colors.json");

const height = {
  sm: "16px",
  md: "16px",
  lg: "24px",
  xl: "48px",
};

// picked these values with a little help from
// https://css-tricks.com/optimizing-large-scale-displays/
const screens = {
  // everything smaller is considered "mobile"
  tablet: "768px", // smaller laptops may also fall into this group
  desktop: "1366px", // size of 13" mac laptop
  wide: "1920px", // large-ish screen
  huge: "2560px", // 27" hi res monitors
};

const spacing = {
  // named spacing scale, hopefully can help reduce the number of variations being used?
  none: "0",
  "3xs": `${2 / 16}rem`, // tw 0.5
  "2xs": `${4 / 16}rem`, // tw 1
  xs: `${8 / 16}rem`, // tw 2
  sm: `${16 / 16}rem`, // tw 4
  md: `${24 / 16}rem`, // tw 6
  lg: `${36 / 16}rem`, // tw 8
  xl: `${64 / 16}rem`, // tw 16
  "2xl": `${96 / 16}rem`, // tw 24
  "3xl": `${128 / 16}rem`, // tw 32

  13: "3.25rem",
  14: "3.5rem",
  52: "13rem",
  54: "13.5rem",
  56: "14rem",
  60: "15rem",
  64: "16rem",
  72: "18rem",
  80: "20rem",
  96: "24rem",
};

const margin = {
  "-05": "-0.05rem",
};

const maxHeight = {
  0: "0",
  "1/4": "25%",
  "1/2": "50%",
  "3/4": "75%",
};

const width = {
  "1/7": "14.285714285714286%",
  "2/7": "28.571428571428571%",
};

const zIndex = {
  60: 60,
  70: 70,
  80: 80,
  90: 90,
  100: 100,
};

module.exports = {
  colors,
  height,
  screens,
  spacing,
  margin,
  maxHeight,
  width,
  zIndex,
};
