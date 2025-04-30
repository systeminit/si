import { Ref, unref } from "vue";
import { Toggle } from "./logic_composables/toggle_containers";

/**
 * Specify the height of a collapsed grid when its closed (e.g. just enough to show a header)
 * When its open use the fractional unit so it grows to the available size, sharing the remaining space with other open grid items
 */
export const gridCollapseStyle = (open: boolean | Ref<boolean, boolean>) =>
  unref(open) ? "1fr" : "1.75em";

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
