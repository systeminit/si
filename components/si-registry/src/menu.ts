import { registry } from "./registry";
import { SchematicKind, RegistryEntryUiMenuItem } from "./registryEntry";

import _ from "lodash";

export interface Category {
  kind: "category";
  name: string;
  items: MenuItem[];
}

export interface Item {
  kind: "item";
  name: string;
  entityType: string;
}

export type MenuItem = Category | Item;

export interface MenuList {
  list: MenuItem[];
}

export interface EntityMenuFilters {
  schematicKind: SchematicKind;
  rootEntityType: string;
}

export function entityMenu(filter: EntityMenuFilters): MenuList {
  const list: MenuList["list"] = [];

  for (const schema of Object.values(registry)) {
    if (schema.entityType == "leftHandPath") {
      continue;
    }
    if (schema.ui.hidden) {
      continue;
    }
    const menuEntry = _.find(
      schema.ui.menu,
      (menuEntry: RegistryEntryUiMenuItem) => {
        if (menuEntry.rootEntityTypes) {
          return (
            menuEntry.schematicKind == filter.schematicKind &&
            _.includes(menuEntry.rootEntityTypes, filter.rootEntityType)
          );
        } else {
          return menuEntry.schematicKind == filter.schematicKind;
        }
      },
    );
    if (!menuEntry) {
      continue;
    }

    let currentList = list;
    for (const category of menuEntry.menuCategory) {
      const existingIndex = _.findIndex(currentList, (i) => {
        return i.name == category && i.kind == "category";
      });
      if (existingIndex == -1) {
        const newLength = currentList.push({
          kind: "category",
          name: category,
          items: [],
        });
        // @ts-ignore
        currentList = currentList[newLength - 1].items;
      } else {
        // @ts-ignore
        currentList = currentList[existingIndex].items;
      }
    }
    currentList.push({
      kind: "item",
      name: menuEntry.name,
      entityType: schema.entityType,
    });
    currentList.sort((a, b) => {
      if (a.kind == "category" && a.name == "implementation") {
        return -1;
      } else {
        const aName = a.name;
        const bName = b.name;
        return aName.localeCompare(bName);
      }
    });
  }
  list.sort((a, b) => {
    if (a.kind == "category" && a.name == "implementation") {
      return -1;
    } else if (b.kind == "category" && b.name == "implementation") {
      return 1;
    } else {
      const aName = a.name;
      const bName = b.name;
      return aName.localeCompare(bName);
    }
  });

  return { list };
}
