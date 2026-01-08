// Our old regex here prevented usage of accents in names
// ^[a-zA-Z0-9-.,_@/+ ]*$/;

// General text input - allows letters, numbers, spaces, and common punctuation
// Excludes characters that enable URLs: . / @
export const ALLOWED_INPUT_REGEX = /^[0-9A-Za-zÀ-ÖØ-öø-ÿĀ-ỹ\s',_+()-]*$/;

// Domain-friendly text input - like ALLOWED_INPUT_REGEX but allows dots (.) for domain names
// Suitable for display names and descriptions that may contain domain names
// Still excludes URL-enabling characters like / and @
export const DOMAIN_FRIENDLY_INPUT_REGEX =
  /^[0-9A-Za-zÀ-ÖØ-öø-ÿĀ-ỹ\s',_+().-]*$/;

// Name fields - more restrictive, only letters, spaces, hyphens, apostrophes
// Prevents URLs and domain names from being entered
export const NAME_REGEX = /^[0-9A-Za-zÀ-ÖØ-öø-ÿĀ-ỹ\s'-]*$/;

// Nickname/username fields - more permissive than NAME_REGEX
// Allows common username characters: letters, numbers, spaces, and punctuation
// Includes: - _ ' + & . ! # ( )
// Still excludes URL-enabling characters: @ / :
export const NICKNAME_REGEX = /^[0-9A-Za-zÀ-ÖØ-öø-ÿĀ-ỹ\s'_+&.!#()-]*$/;

// Max length constants for input validation
export const MAX_LENGTH_STANDARD = 256; // For user profile fields and workspace displayName
export const MAX_LENGTH_EXTENDED = 512; // For workspace description

// Detects common URL patterns to explicitly reject
export const URL_DETECTION_REGEX =
  /(?:https?:\/\/|www\.|[a-z0-9-]+\.(com|org|net|io|co|dev|app|ai|uk|edu|gov|xyz|info|biz|me|tv|online|site|tech|cloud|store|blog))/i;

export const ALLOWED_URL_REGEX =
  "^https?://([\\da-z.-]+)(:\\d+)?(/[\\w .-]*)*/?$";

// GitHub username - allows empty string OR valid username
export const GITHUB_USERNAME_REGEX =
  /^$|^[a-z\d](?:[a-z\d]|-(?=[a-z\d])){0,38}$/i;
// Discord username - allows empty string OR valid username/tag
export const DISCORD_TAG_REGEX =
  /^$|^(?!(discord|here|everyone))(((?!.*\.\.)(([\w.]{2,32})))|[^@#:]{2,32}#[\d]{4})$/i;
