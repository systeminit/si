const colors = {
  //
  // # Neutral
  //
  // Probably the most frequently used colors in the palette. These colors are
  // used for backgrounds, text colors, strokes, separators, dialogs, menus,
  // modals etc
  neutral: {
    50: "#FAFAFA",
    100: "#F5F5F5",
    200: "#E5E5E5",
    300: "#D4D4D4",
    400: "#A3A3A3",
    500: "#737373",
    600: "#525252",
    700: "#404040",
    800: "#333333",
    900: "#262626",
  },
  //
  // # Action
  //
  // The action is used across all the interactive elements in the product such
  // as buttons, links, inputs, active states, highlights etc.
  action: {
    50: "#EFF6FE",
    100: "#E2F3FE",
    200: "#B2E0FF",
    300: "#B2E0FF",
    400: "#0E9BFF",
    500: "#2F80ED",
    600: "#1975DC",
    700: "#3B65A8",
    800: "#395080",
    900: "#424F6B",
  },
  //
  // # Success
  //
  // These colors tend to convey positive emotions. Generally used across
  // success and completed states.
  success: {
    50: "#F0FDF4",
    100: "#DCFCE7",
    200: "#BBF7D0",
    300: "#86EFAC",
    400: "#4ADE80",
    500: "#22C55E",
    600: "#16A34A",
    700: "#15803D",
    800: "#166534",
    900: "#14532D",
  },
  //
  // # Warning
  //
  // Colors that conventionally intended to convey the feeling of caution.
  // Generally used across warning states
  warning: {
    50: "#FFFBEB",
    100: "#FEF3C7",
    200: "#FDE68A",
    300: "#FCD34D",
    400: "#FBBF24",
    500: "#F59E0B",
    600: "#D97706",
    700: "#B45309",
    800: "#92400E",
    900: "#78350F",
  },
  //
  // # Destructive
  //
  // Colors that conventionally intended to convey feelings of urgency or even
  // negativity. Generally used across error states and for destructive actions
  destructive: {
    50: "#FEF2F2",
    100: "#FEE2E2",
    200: "#FECACA",
    300: "#FCA5A5",
    400: "#F87171",
    500: "#EF4444",
    600: "#DC2626",
    700: "#B91C1C",
    800: "#991B1B",
    900: "#7F1D1D",
  },
  //
  // # Shade
  //
  // Shades are a spectrum of black and white. Use freely.
  shade: {
    100: "#000000",
    0: "#FFFFFF",
  },

  //
  // TODO(fnichol): Pre-existing colors from prior iterations. Should we
  // consider removing these to limit our color palette choices, at least as
  // far as outdated customized colors goes? Note that as of this note, we have
  // an older interface iteration co-existing with the current iteration
  // interface so this is here in an attempt to preserve backwards
  // compatibility. Also note that I had to replace the `success` and `warning`
  // keys as they collided with new color names.
  //
  primary: "#151B1E",
  secondary: "#ECEFF1",
  accent: "#607D8B",
  error: "#FF5252",
  info: "#2196F3",
  // success: "#4CAF50",
  // warning: "#FB8C00",
  black: "#000000",
  selectordark: "#1B1B1B",
  selector1: "#343B3F",
};

const height = {
  sm: "16px",
  md: "16px",
  lg: "24px",
  xl: "48px",
};

const screens = {
  sm: "640px",
  md: "768px",
  lg: "1024px",
  xl: "1280px",
};

const spacing = {
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

exports.colors = colors;
exports.spacing = spacing;
exports.screens = screens;
exports.margin = margin;
exports.width = width;
exports.height = height;
exports.maxHeight = maxHeight;
exports.zIndex = zIndex;
