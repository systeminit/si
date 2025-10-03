// Our old regex here prevented usage of accents in names
// ^[a-zA-Z0-9-.,_@/+ ]*$/;

// This one is much more permissive but still might not be quite right
// We probably want to figure out exactly what characters to include or not include
// Useful here - https://en.wikipedia.org/wiki/List_of_Unicode_characters
// TODO - update this REGEX to be even better!
export const ALLOWED_INPUT_REGEX = /^[0-9A-Za-zÀ-ÖØ-öø-ÿĀ-ỹ-.,_@/+ ]*$/;

export const ALLOWED_URL_REGEX =
  "^https?://([\\da-z.-]+)(:\\d+)?(/[\\w .-]*)*/?$";
