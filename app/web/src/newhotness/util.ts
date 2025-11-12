import * as _ from "lodash-es";
import { Ref, unref } from "vue";
import {
  IconNames,
  BRAND_COLOR_FILTER_HEX_CODES,
} from "@si/vue-lib/design-system";
import {
  AttributeTree,
  AttributeValue,
} from "@/workers/types/entity_kind_types";
import { AttributePath } from "@/api/sdf/dal/component";
import { Toggle } from "./logic_composables/toggle_containers";

export type BrandColorKey = keyof typeof BRAND_COLOR_FILTER_HEX_CODES;

export const getAssetColor = (name: string) => {
  const lowerCaseName = name.toLowerCase();

  const key = (
    Object.keys(BRAND_COLOR_FILTER_HEX_CODES) as BrandColorKey[]
  ).find((key) => lowerCaseName.startsWith(key.toLowerCase()));

  return key
    ? BRAND_COLOR_FILTER_HEX_CODES[key]
    : BRAND_COLOR_FILTER_HEX_CODES.Custom; // fallback to Custom
};

export const pickBrandIconByString = (name: string): IconNames => {
  if (name.toLowerCase().includes("aws")) return "logo-aws";
  else if (name.toLowerCase().includes("coreos")) return "logo-coreos";
  else if (name.toLowerCase().includes("docker")) return "logo-docker";
  else if (name.toLowerCase().includes("fastly")) return "logo-fastly";
  else if (name.toLowerCase().includes("kubernetes")) return "logo-k8s";
  else if (name.toLowerCase().includes("hetzner")) return "logo-hetzner";
  else if (
    name.toLowerCase().includes("microsoft") ||
    name.toLowerCase().includes("azure")
  )
    return "logo-azure";
  else if (
    name.toLowerCase().includes("digitalocean") ||
    name.toLowerCase().includes("digital ocean")
  )
    return "logo-digital-ocean";
  else return "logo-si";
};

/**
 * Specify the height of a collapsed grid when its closed (e.g. just enough to show a header)
 * When its open use the fractional unit so it grows to the available size, sharing the remaining space with other open grid items
 */
export const gridCollapseStyle = (open: boolean | Ref<boolean, boolean>) =>
  unref(open) ? "1fr" : "2.5em";

/**
 * Generates the styles for a vertical grid of collapsing panels
 *
 * @param gridStates a list of open / closed states that represent the grid items
 * @returns { gridTemplateRows: string of sizes in the grid order you passed in }
 */
export const collapsingGridStyles = (
  gridStates: (Pick<Toggle, "open"> | undefined)[],
): Record<string, string> => {
  // NOTE: the optional `gs` and coalesce is for rendering states before template refs are instantiated (that is what the | undefined above represents)
  const sizes = gridStates.map((gs): string =>
    gridCollapseStyle(gs?.open ?? true),
  );
  return {
    gridTemplateRows: sizes.join(" "),
  };
};

// Used in the component page vue components
export const findAvsAtPropPath = (data: AttributeTree, parts: string[]) => {
  const path = parts.join("/");
  const propId = Object.keys(data.props).find((pId) => {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const p = data.props[pId]!;
    if (p.path === path) return true;
    return false;
  });
  if (!propId) return null;
  const avIds = Object.keys(data.attributeValues).filter((avId) => {
    const a = data.attributeValues[avId];
    if (a?.propId === propId) return true;
    return false;
  });
  if (avIds.length === 0) return null;
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const prop = data.props[propId]!;
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const attributeValues = avIds.map((avId) => data.attributeValues[avId]!);
  return { prop, attributeValues };
};

export const findAttributeValueInTree = (
  tree: AttributeTree,
  targetPath: AttributePath,
): { attributeValue: AttributeValue; avId: string } | null => {
  const pathStr = targetPath.toString();

  for (const [avId, av] of Object.entries(tree.attributeValues)) {
    if (av.path === pathStr) {
      return { attributeValue: av, avId };
    }
  }
  return null;
};

export function objectKeys<T extends object>(obj: T): Array<keyof T> {
  return Object.keys(obj) as Array<keyof T>;
}

export const openWorkspaceMigrationDocumentation = () => {
  globalThis.window?.open(
    "https://docs.systeminit.com/explanation/workspace-migration",
    "_blank",
  );
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnyFn = (...args: any[]) => any;

export interface MemoizeDebouncedFn<F extends AnyFn>
  extends _.DebouncedFunc<F> {
  (...args: Parameters<F>): ReturnType<F> | undefined;
  flush: (...args: Parameters<F>) => ReturnType<F> | undefined;
  cancel: (...args: Parameters<F>) => void;
}
/**
 * Debounce based on args to the fn
 */
export function memoizeDebounce<F extends AnyFn>(
  func: F,
  wait = 0,
  options: _.DebounceSettings = {},
  resolver?: (...args: Parameters<F>) => unknown,
): MemoizeDebouncedFn<F> {
  const dbMemo = _.memoize<(...args: Parameters<F>) => _.DebouncedFunc<F>>(
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    (..._args: Parameters<F>) => _.debounce(func, wait, options),
    resolver,
  );

  function wrappedFn(
    this: MemoizeDebouncedFn<F>,
    ...args: Parameters<F>
  ): ReturnType<F> | undefined {
    return dbMemo(...args)(...args);
  }

  const flush: MemoizeDebouncedFn<F>["flush"] = (...args) => {
    return dbMemo(...args).flush();
  };

  const cancel: MemoizeDebouncedFn<F>["cancel"] = (...args) => {
    return dbMemo(...args).cancel();
  };

  wrappedFn.flush = flush;
  wrappedFn.cancel = cancel;

  return wrappedFn;
}

/**
 * Escapes a string segment for use in a JSON Pointer (RFC 6901).
 * JSON Pointers use '~' as an escape character:
 * - '~' must be encoded as '~0'
 * - '/' must be encoded as '~1'
 *
 * @param segment - The string segment to escape
 * @returns The escaped segment safe for use in a JSON Pointer path
 *
 * @example
 * escapeJsonPointerSegment("test/paul") // returns "test~1paul"
 * escapeJsonPointerSegment("a~b") // returns "a~0b"
 * escapeJsonPointerSegment("test/~foo") // returns "test~1~0foo"
 */
export function escapeJsonPointerSegment(segment: string): string {
  // Order matters: escape ~ first, then /
  return segment.replace(/~/g, "~0").replace(/\//g, "~1");
}
