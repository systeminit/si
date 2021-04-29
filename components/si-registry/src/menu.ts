import { registry } from "./registry";
import { RegistryEntry, MenuCategory, SchematicKind } from "./registryEntry";

import _ from "lodash";

export interface MenuItem {
  entityType: string;
  displayName: string;
}

export interface MenuCategoryItem {
  name: string;
  items: MenuItem[];
}

export interface MenuList {
  list: MenuCategoryItem[];
}

function enumKeys<O extends object, K extends keyof O = keyof O>(obj: O): K[] {
  return Object.values(obj).filter((k) => Number.isNaN(+k)) as K[];
}

export function entityMenu(filter: SchematicKind[]): MenuList {
  const list: MenuList["list"] = [];
  const categories = enumKeys(MenuCategory).sort();
  for (const category of categories) {
    const items: MenuCategoryItem["items"] = [];
    list.push({ name: category, items });
  }
  for (const schema of Object.values(registry)) {
    if (schema.entityType == "leftHandPath") {
      continue;
    }
    if (schema.ui.hidden) {
      continue;
    }
    // Let us never speak of this again.
    if (filter.length > 0) {
      if (schema.ui.schematicKinds) {
        // @ts-ignore
        let skipThisEntry = true;
        for (const s of schema.ui.schematicKinds) {
          for (const f of filter) {
            if (f == s) {
              skipThisEntry = false;
            }
          }
        }
        if (skipThisEntry) {
          continue;
        }
      }
    }
    const mcIndex = _.findIndex(list, ["name", schema.ui.menuCategory]);
    let displayName = schema.entityType;
    if (schema.ui.menuDisplayName) {
      displayName = schema.ui.menuDisplayName;
    }
    list[mcIndex].items.push({
      entityType: schema.entityType,
      displayName,
    });
  }

  let reducedList: MenuList["list"] = [];
  for (const mc of list) {
    if (mc.items.length) {
      reducedList.push(mc);
    }
  }

  for (const mc of list) {
    mc.items = _.sortBy(mc.items, ["displayName"]);
  }

  reducedList = _.sortBy(reducedList, ["name"]);

  return { list: reducedList };
}
