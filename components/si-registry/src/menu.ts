import { registry } from "./registry";
import { RegistryEntry, MenuCategory } from "./registryEntry";

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

export function entityMenu(deploymentNodes?: boolean, implementationNodes?: boolean): MenuList {
  
  let list: MenuList["list"] = [];
  const categories = enumKeys(MenuCategory).sort();
  for (const category of categories) {
    const items: MenuCategoryItem["items"] = [];
    list.push({ name: category, items });
  }
  for (const schema of Object.values(registry)) {
    if (schema.entityType == "leftHandPath") {
      continue;
    }
    if (!schema.ui.hidden) {
      const mc = _.find(list, ["name", schema.ui.menuCategory]);
      
      if (deploymentNodes && schema.ui.superNode) {
        let displayName = schema.entityType;
        // @ts-ignore
        if (schema.ui.menuDisplayName) {
          // @ts-ignore
          displayName = schema.ui.menuDisplayName;
        }
        mc.items.push({
          entityType: schema.entityType,
          displayName,
        });
      }

      if (implementationNodes && !schema.ui.superNode) {
        let displayName = schema.entityType;
        // @ts-ignore
        if (schema.ui.menuDisplayName) {
          // @ts-ignore
          displayName = schema.ui.menuDisplayName;
        }
        mc.items.push({
          entityType: schema.entityType,
          displayName,
        });
      }
    }
  }

  let reducedList: MenuList["list"] = [];
  for (const mc of list) {
    if (mc.items.length) {
      reducedList.push(mc)
    }
  }

  for (const mc of list) {
    mc.items = _.sortBy(mc.items, ["displayName"]);
  }

  reducedList = _.sortBy(reducedList, ["name"]);
  return { list: reducedList };

}
