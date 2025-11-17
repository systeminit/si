import { useQuery } from "@tanstack/vue-query";
import { MaybeRefOrGetter, Ref, toValue } from "vue";
import { Fzf } from "fzf";
import { bifrostQueryAttributes, useMakeKey } from "@/store/realtime/heimdall";
import { ComponentInList, EntityKind } from "@/workers/types/entity_kind_types";
import { QueryAttributesTerm } from "@/workers/types/dbinterface";
import { assertUnreachable } from "@/utils/assertunreachable";
import { useContext } from "./context";
import { computedAsyncDebounce } from "./async";
import { Context } from "../types";

/**
 * A reactive component search.
 *
 *     const searchString = ref<string>();
 *     useComponentSearch()
 *
 * If a search has not yet happened, or if the list of components is undefined, this function
 * will return undefined.
 *
 * @param searchString - The search string (which can be reactive). If the string is undefined
 *                       or empty, all components will be returned.
 * @param componentList - The list of components to search through (which can be reactive).
 */
export function useComponentSearch(
  searchString: MaybeRefOrGetter<string | undefined>,
  componentsRef: MaybeRefOrGetter<ComponentInList[] | undefined>,
): Ref<ComponentInList[] | undefined> {
  // This listens for changes to attributes so the search will be re-run if they change.
  const ctx = useContext();
  const key = useMakeKey();
  // Use tanstack `useQuery` so we can bust the cache of QueryAttributes whenever an
  // AttributeTree is updated. We don't actuall;y do the query in here though, we just return
  // a function that *can* do the query!
  const attributeTreesUpdatedAt = useQuery({
    queryKey: key(EntityKind.QueryAttributes),
    queryFn: () => new Date(),
  });

  // This throttles: If the search string or components change are made while a query is
  // running, computedAsync will wait until the current query is finished before re-running.
  return computedAsyncDebounce(async () => {
    // Just mentioning this will cause us to recompute when any attribute trees update.
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    attributeTreesUpdatedAt.data.value;
    // If components is undefined, return undefined (means there's nothing to search yet).
    const components = toValue(componentsRef);
    if (!components) return undefined;

    // If search string is undefined or empty, return all components.
    const searchTerms = parseSearchTerms(toValue(searchString));
    if (!searchTerms) return components;

    // Return search results!
    const comps = await search(components, searchTerms);
    return comps;
  });

  async function search(
    components: ComponentInList[],
    term: SearchTerms,
  ): Promise<ComponentInList[]> {
    // NOTE: this does an exhaustiveness check
    switch (term.op) {
      case "not": {
        // Find the matches, then pick everything else
        const removeComponents = new Set(
          (await search(components, term.condition)).map((c) => c.id),
        );
        return components.filter((c) => !removeComponents.has(c.id));
      }
      case "and": {
        // Just narrow down the results by applying each condition.
        for (const condition of term.conditions) {
          components = await search(components, condition);
        }
        return components;
      }
      case "or": {
        // Figure out which things match; but maintain the order of the individual searches
        const results = new Set<ComponentInList>();
        for (const condition of term.conditions) {
          // Add results in the order they were defined (unless they are duplicates)
          for (const component of await search(components, condition)) {
            results.add(component);
          }
        }
        return Array.from(results);
      }
      case "exact": {
        // Make sure the term is an exact match for name/schemaName/schemaCategory/id
        // TODO AWS::EC2::Instance vs. Instance: should both work?
        // TODO support *
        return components.filter(
          (c) =>
            c.name.localeCompare(term.value) === 0 ||
            c.schemaCategory.localeCompare(term.value) === 0 ||
            c.schemaName.localeCompare(term.value) === 0 ||
            c.id.localeCompare(term.value) === 0,
        );
      }
      case "startsWith": {
        // Make sure the term is an exact match for name/schemaName/schemaCategory/id
        // TODO AWS::EC2::Instance vs. Instance: should both work?
        // TODO support *
        const value = term.value.toLowerCase();
        return components.filter(
          (c) =>
            c.name.toLowerCase().startsWith(value) ||
            c.schemaCategory.toLowerCase().startsWith(value) ||
            c.schemaName.toLowerCase().startsWith(value) ||
            c.id.toLowerCase().startsWith(value),
        );
      }
      case "fuzzy": {
        // Regular fuzzy search across all fields
        const fzf = new Fzf(components, {
          casing: "case-insensitive",
          selector: (c) =>
            `${c.name} ${c.schemaCategory} ${c.schemaName} ${c.id}`,
        });
        return fzf.find(term.value).map((fz) => fz.item);
      }
      case "attr": {
        // Query to find the component IDs matching this attr, then use that to narrow the components
        const startTerms = term.startsWith.map((value) => ({
          key: term.key,
          value,
          op: "startsWith" as const,
        }));
        const exactTerms = term.exact.map((value) => ({
          key: term.key,
          value,
          op: "exact" as const,
        }));

        // If we get a key with no value (key:), we push in a single empty string, which will match
        // all components with that key set to anything
        if (exactTerms.length === 0 && startTerms.length === 0) {
          startTerms.push({
            key: term.key,
            value: "",
            op: "startsWith" as const,
          });
        }

        const componentIds = new Set(
          await queryAttributes(ctx, [...startTerms, ...exactTerms]),
        );
        return components.filter((c) => componentIds.has(c.id));
      }
      default:
        return assertUnreachable(term);
    }
  }
}

/**
 * Function that can be used to reactively query attributes of all components.
 */
export async function queryAttributes(
  ctx: Context,
  terms: MaybeRefOrGetter<QueryAttributesTerm[]>,
) {
  return await bifrostQueryAttributes(
    ctx.workspacePk.value,
    ctx.changeSetId.value,
    toValue(terms),
  );
}

/**
 * Parse search string into an expression tree supporting boolean operators, (), attr: and "exact"
 */
export function parseSearchTerms(search: string): SearchTerms;
export function parseSearchTerms(
  search: string | undefined,
): SearchTerms | undefined;
export function parseSearchTerms(search: string | undefined) {
  if (search === undefined) return undefined;
  return new SearchParser(search).parse();
}

/**
 * Search string, split into terms.
 *
 *     {
 *       op: "and",
 *       conditions: [
 *          { op: "fuzzy", value: "MyComponent" },
 *          { op: "attr", key: "schema", value: "AWS::EC2::Subnet" },
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
  | { op: "attr"; key: string; exact: string[]; startsWith: string[] };

class SearchParser {
  constructor(private search: string) {}
  private index = 0;

  parse(): SearchTerms {
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
    else return conditions[0] ?? { op: "fuzzy", value: "" };
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
      const startsWith: string[] = [];
      const exact: string[] = [];

      // Always get first string after :, and if we hit an OR separator, get the next string
      do {
        // If the string starts with ", we treat it as an exact match value (unless it's not closed, read below)
        if (this.consume('"')) {
          const attrString = this.consumeUntil(['"']);
          // If we get a closing quote, push it to exact matches otherwise, we treat it as a startsWith to improve UX while typing
          if (this.consume('"')) {
            exact.push(attrString);
          } else {
            // when the user starts typing an exact value, empty strings would match everything and re-add items, which is jarring
            startsWith.push(attrString);
          }
        } else {
          // No quotes, it will match any value that starts with this string
          const val = this.consumeUntil([
            " ",
            "(",
            ")",
            "&",
            "!",
            '"',
            "|",
            ",",
          ]);
          startsWith.push(val);
        }
      } while (this.consume("|") || this.consume(","));

      return {
        op: "attr",
        key: value,
        // Drop empty strings that will be generated by trailing commas or |s, or during the typing process of an exact match.
        // since empty strings match everything, they may jarringly re-add items that were filtered out already
        startsWith: startsWith.filter((v) => v !== ""),
        exact,
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
