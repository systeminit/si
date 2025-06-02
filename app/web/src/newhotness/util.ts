import { Ref, unref } from "vue";
import { IconNames } from "@si/vue-lib/design-system";
import { AttributeTree } from "@/workers/types/entity_kind_types";
import { Toggle } from "./logic_composables/toggle_containers";

export const getAssetIcon = (name: string) => {
  const icons = {
    AWS: "logo-aws",
    "AWS EC2": "logo-aws",
    CoreOS: "logo-coreos",
    Docker: "logo-docker",
    Kubernetes: "logo-k8s",
  } as Record<string, string>;

  let icon = icons[name];

  if (!icon) {
    for (const k in icons) {
      if (name.includes(k)) {
        icon = icons[k];
      }
    }
  }

  return (icon || "logo-si") as IconNames; // fallback to SI logo
};

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
