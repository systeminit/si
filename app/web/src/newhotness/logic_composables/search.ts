/**
 * Search string, split into terms.
 *
 *     {
 *       op: "and",
 *       conditions: [
 *          { op: "fuzzy", value: "MyComponent" },
 *          { op: "known", key: "schema", value: "AWS::EC2::Subnet" },
 *          { op: "attr", key: "vpcId", value: "vpc-123" },
 *          { op: "exact", value: "AWS::EC2::Subnet" },
 *       ]
 *     }
 *
 */
export type SearchTerms =
  | {
      op: "not";
      condition: SearchTerms;
    }
  | {
      op: "and" | "or";
      conditions: SearchTerms[];
    }
  | { op: "fuzzy" | "exact" | "startsWith"; value: string }
  | { op: "attr"; key: string; values: string[] };

export function parseSearch(search: string): SearchTerms | undefined {
  return new SearchParser(search).parse();
}

class SearchParser {
  constructor(private search: string) {}
  private index = 0;

  parse(): SearchTerms | undefined {
    // parseCondition() stops at end paren, but will consume anything else.
    // At the top level, we want to make sure we consume the entire search string. If we find
    // stray end parens, we explicitly ignore them here and continue parsing more conditions.
    const conditions: SearchTerms[] = [];
    do {
      const condition = this.parseCondition();
      if (condition) conditions.push(condition);
    } while (this.consume(")")); // We eat stray end parens for breakfast

    // If we're not at end of string, something is wrong with the parser (it's designed to parse
    // the entire search string, even if it's malformed).
    if (!this.eof()) {
      throw new Error(
        `Search parse error at ${this.index}: ${this.search.slice(this.index)}`,
      );
    }

    // If there were extra parens, we want to honor all the conditions, so stuff them together
    // in an "&".
    if (conditions.length > 1) return { op: "and", conditions };
    else return conditions[0];
  }

  /**
   * Parse a full search condition (|, &/space, !, (), "", value, attr:value).
   *
   * Stops at ) or end of string.
   *
   * @return the condition, or undefined if we're at ) or end of string.
   */
  parseCondition(): SearchTerms | undefined {
    return this.parseOrCondition();
  }

  /**
   * Parse an OR expression (|, &/space, !, (), "", value, attr:value)
   *
   * @return the condition, or undefined if there is no conditoin and we're at ) or end of string.
   */
  parseOrCondition(): SearchTerms | undefined {
    const conditions: SearchTerms[] = [];
    do {
      const condition = this.parseAndCondition();
      if (condition) conditions.push(condition);
    } while (this.consume("|"));

    if (conditions.length > 1) return { op: "or", conditions };
    else return conditions[0];
  }

  /**
   * Parse an AND expression (&/space, !, (), "", value, attr:value).
   *
   * @return the condition, or undefined if we're at |, ), or end of string.
   */
  parseAndCondition(): SearchTerms | undefined {
    const conditions: SearchTerms[] = [];
    do {
      const condition = this.parseTerm();
      if (condition) conditions.push(condition);
    } while (this.consume(" ") || this.consume("&"));

    if (conditions.length > 1) return { op: "and", conditions };
    else return conditions[0];
  }

  /**
   * Parse a single term (!, (), "", value, attr:value).
   *
   * @return the term, or undefined if we're at |, &, ), space, or end of string.
   */
  parseTerm(): SearchTerms | undefined {
    // Parse parens as a single term (...)
    if (this.consume("(")) {
      const condition = this.parseCondition();
      this.consume(")"); // Don't care if it's actually there; end of string closes all parens.
      return condition;
    }

    // Parse !expression
    if (this.consume("!")) {
      // Skip whitespace (support ! <term>)
      // eslint-disable-next-line no-empty
      while (this.consume(" ")) {}
      const condition = this.parseTerm();
      return condition ? { op: "not", condition } : undefined;
    }

    // Parse quoted term "..."
    if (this.consume('"')) {
      const value = this.consumeUntil(['"']);
      if (this.consume('"')) {
        return { op: "exact", value };
      } else {
        // Unclosed quotes should still show the partial match!
        return { op: "startsWith", value };
      }
    }

    //
    // Parse value or attr:value
    //

    // Read everything up to the next special character or : (first word)
    // If this is attr:value:str, this will only read "attr" and we'll read "value:str" next.
    const value = this.consumeUntil([" ", "(", ")", "&", "|", "!", '"', ":"]);
    // If there's no term, we're at a special character; let a parent handle it.
    if (value === "") return undefined;

    // If there is a ":", parse it as an attribute (take the stuff after the :, which might
    // have other colons in it).
    if (this.consume(":")) {
      const values: string[] = [];

      // Always get first string after :, and if we hit an OR separator, get the next string
      do {
        const val = this.consumeUntil([" ", "(", ")", "&", "!", '"', "|", ","]);
        // Drop empty strings that will be generated by trailing commas or |s
        if (values.length === 0 || val !== "") {
          values.push(val);
        }
      } while (this.consume("|") || this.consume(","));

      return {
        op: "attr",
        key: value,
        values,
      };
    }

    return { op: "fuzzy", value };
  }

  consume(char: string) {
    const current = this.search[this.index];
    if (current === char) {
      this.index++;
      return this.search[this.index - 1];
    }
    return undefined;
  }

  consumeUntil(chars: string[]) {
    const start = this.index;
    // Stop at end of string or one of the included character.
    while (!(this.eof() || chars.includes(this.search[this.index] ?? ""))) {
      this.index++;
    }
    return this.search.slice(start, this.index);
  }

  eof() {
    return this.index >= this.search.length;
  }
}
