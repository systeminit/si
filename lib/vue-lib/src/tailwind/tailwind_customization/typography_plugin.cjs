/* eslint-disable import/no-extraneous-dependencies */
const plugin = require("tailwindcss/plugin");

const siTypographyTailwindPlugin = plugin(({ addUtilities }) => {
  addUtilities({
    //
    // Type Scale / Light / Regular
    //
    ".type-regular-3xl": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "1.875rem",
      "line-height": "2.25rem",
    },
    ".type-regular-2xl": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "1.5rem",
      "line-height": "2rem",
    },
    ".type-regular-xl": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "1.25rem",
      "line-height": "1.75rem",
    },
    ".type-regular-lg": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "1.125rem",
      "line-height": "1.75rem",
    },
    ".type-regular-base": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "1rem",
      "line-height": "1.5rem",
    },
    ".type-regular-sm": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "0.875rem",
      "line-height": "1.25rem",
    },
    ".type-regular-xs": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "0.75rem",
      "line-height": "1rem",
    },
    //
    // Type Scale / Light / Medium
    //
    ".type-medium-3xl": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 500,
      "font-size": "1.875rem",
      "line-height": "2.25rem",
    },
    ".type-medium-2xl": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 500,
      "font-size": "1.5rem",
      "line-height": "2rem",
    },
    ".type-medium-xl": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 500,
      "font-size": "1.25rem",
      "line-height": "1.75rem",
    },
    ".type-medium-lg": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 500,
      "font-size": "1.125rem",
      "line-height": "1.75rem",
    },
    ".type-medium-base": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 500,
      "font-size": "1rem",
      "line-height": "1.5rem",
    },
    ".type-medium-sm": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 500,
      "font-size": "0.875rem",
      "line-height": "1.25rem",
    },
    ".type-medium-xs": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 500,
      "font-size": "0.75rem",
      "line-height": "1rem",
    },
    //
    // Type Scale / Light / Bold
    //
    ".type-bold-3xl": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 700,
      "font-size": "1.875rem",
      "line-height": "2.25rem",
    },
    ".type-bold-2xl": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 700,
      "font-size": "1.5rem",
      "line-height": "2rem",
    },
    ".type-bold-xl": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 700,
      "font-size": "1.25rem",
      "line-height": "1.75rem",
    },
    ".type-bold-lg": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 700,
      "font-size": "1.125rem",
      "line-height": "1.75rem",
    },
    ".type-bold-base": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 700,
      "font-size": "1rem",
      "line-height": "1.5rem",
    },
    ".type-bold-sm": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 700,
      "font-size": "0.875rem",
      "line-height": "1.25rem",
    },
    ".type-bold-xs": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 700,
      "font-size": "0.75rem",
      "line-height": "1rem",
    },
    //
    // Type Scale / Light / Italic
    //
    ".type-italic-3xl": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "1.875rem",
      "line-height": "2.25rem",
      "font-style": "italic",
    },
    ".type-italic-2xl": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "1.5rem",
      "line-height": "2rem",
      "font-style": "italic",
    },
    ".type-italic-xl": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "1.25rem",
      "line-height": "1.75rem",
      "font-style": "italic",
    },
    ".type-italic-lg": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "1.125rem",
      "line-height": "1.75rem",
      "font-style": "italic",
    },
    ".type-italic-base": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "1rem",
      "line-height": "1.5rem",
      "font-style": "italic",
    },
    ".type-italic-sm": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "0.875rem",
      "line-height": "1.25rem",
      "font-style": "italic",
    },
    ".type-italic-xs": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "0.75rem",
      "line-height": "1rem",
      "font-style": "italic",
    },
    //
    // Type Scale / Light / Underline
    //
    ".type-underline-3xl": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "1.875rem",
      "line-height": "2.25rem",
      "text-decoration-line": "underline",
    },
    ".type-underline-2xl": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "1.5rem",
      "line-height": "2rem",
      "text-decoration-line": "underline",
    },
    ".type-underline-xl": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "1.25rem",
      "line-height": "1.75rem",
      "text-decoration-line": "underline",
    },
    ".type-underline-lg": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "1.125rem",
      "line-height": "1.75rem",
      "text-decoration-line": "underline",
    },
    ".type-underline-base": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "1rem",
      "line-height": "1.5rem",
      "text-decoration-line": "underline",
    },
    ".type-underline-sm": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "0.875rem",
      "line-height": "1.25rem",
      "text-decoration-line": "underline",
    },
    ".type-underline-xs": {
      "font-family": "Inter, Sans-serif",
      "letter-spacing": "-2%",
      "font-weight": 400,
      "font-size": "0.75rem",
      "line-height": "1rem",
      "text-decoration-line": "underline",
    },
  });
});

module.exports = siTypographyTailwindPlugin;
