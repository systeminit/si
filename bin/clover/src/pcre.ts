// Converts a PCRE pattern from the cloudformation schema into a JS RegExp
// This is not intended to exhaustively convert any PCRE to RegExp; it is designed to be
// simple and Good Enough (TM) to handle the published AWS CF schemas.
export function cfPcreToRegexp(pattern: string) {
  if (pattern in BROKEN_REGEXES) return BROKEN_REGEXES[pattern];

  // Extract flags from the beginning of the pattern
  let flags = "";
  const matchFlags = pattern.match(/^\(\?([ims]+)\)/);
  if (matchFlags) {
    flags += matchFlags[1];
    pattern = pattern.slice(matchFlags[0].length);
  }

  // Go through, looking for ?-style subpatterns we can replace
  const replacements = [];
  for (let index = 0; index < pattern.length; index++) {
    switch (pattern[index]) {
      case "(":
        // We only care about special patterns. All else is fine.
        if (pattern[index + 1] === "?") {
          switch (pattern[index + 2]) {
            // (?:, ?=, ?!, ?<= and ?<! translate just fine)
            case ":":
            case "=":
            case "!":
            case "<":
              break;
            case "-":
              if (pattern.slice(index).startsWith("(?-s:.*)")) {
                replacements.push({
                  start: index,
                  end: index + "(?-s:.*)".length,
                  replacement: "[^\\r\\n]*",
                });
                index += "(?-s:.*)".length - 1;
              } else {
                throw new Error(
                  `Unsupported subpattern: ${pattern.slice(index)}`,
                );
              }
              break;
            default:
              throw new Error(
                `Unsupported subpattern: ${pattern.slice(index)}`,
              );
          }
        }
        break;
      // Skip an extra character if it's an escape code
      case "\\":
        switch (pattern[index + 1]) {
          // \A and \Z match the start and end of a string, respectively
          case "A":
            replacements.push({
              start: index,
              end: index + 2,
              replacement: "^",
            });
            break;
          case "Z":
            replacements.push({
              start: index,
              end: index + 2,
              replacement: "$",
            });
            break;
        }
        index++;
        break;
      case "[":
        // Skip stuff in [] (could be a parenthesis)
        while (index < pattern.length && pattern[index] !== "]") {
          if (pattern[index] === "\\") {
            switch (pattern[index + 1]) {
              // If \p is specified, the "u" flag is required. This makes things more strict, so
              // we don't enable it for everything ...
              case "p":
                if (!flags.includes("u")) flags += "u";
                break;
            }
            index++;
          }
          index++;
        }
        break;
    }
  }

  // Go back through in reverse order, replacing anything that needs it
  while (replacements.length > 0) {
    const { start, end, replacement } = replacements.pop()!;
    pattern = pattern.substring(0, start) + replacement +
      pattern.substring(end);
  }
  return { pattern, flags: flags === "" ? undefined : flags };
}

const BROKEN_REGEXES: Record<
  string,
  { pattern: string; flags?: string } | undefined
> = {
  // The * is redundant, here--it's fairly clear the intent was to limit to 1-1000 characters
  "^[a-zA-Z-0-9-:\\/]*{1,1000}$": {
    pattern: "^[a-zA-Z-0-9-:\\/]{1,1000}$",
    flags: undefined,
  },
  "^[a-zA-Z-0-9-:\\/.]*{1,1000}$": {
    pattern: "^[a-zA-Z-0-9-:\\/.]{1,1000}$",
    flags: undefined,
  },
  "[\\u0020-\\uD7FF\\uE000-\\uFFFD\\uD800\\uDC00-\\uDBFF\\uDFFF\t]*": {
    pattern: "[\\u0020-\\uD7FF\\uE000-\\uFFFD\\uD800\\uDFFF\t]*",
    flags: undefined,
  },
  "[\\u0020-\\uD7FF\\uE000-\\uFFFD\\uD800\\uDC00-\\uDBFF\\uDFFF\\t]*": {
    pattern: "[\\u0020-\\uD7FF\\uE000-\\uFFFD\\uD800\\uDFFF\\t]*",
    flags: undefined,
  },
  "^(?! )[\\p{L}\\p{N}\\p{Z}-_]*(?<! )$": {
    pattern: "^(?! )[\\p{L}\\p{N}\\p{Z}\\-_]*(?<! )$",
    flags: undefined,
  },
  "^(?!(?i)aws)[A-Za-z0-9]{2,64}::[A-Za-z0-9]{2,64}::[A-Za-z0-9]{2,64}$": {
    pattern: "^(?!aws)[A-Za-z0-9]{2,64}::[A-Za-z0-9]{2,64}::[A-Za-z0-9]{2,64}$",
    flags: "i",
  },
  // I don't even
  "[ -퟿-�𐀀-􏿿\r\n\t]*": undefined,
  // This doesn't compile--says it's an invalid property. Seems unsupported.
  "[\\p{Graph}\\x20]*": undefined,
};
