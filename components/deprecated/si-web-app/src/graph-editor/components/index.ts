import Rete from "rete";

import { NumComponent } from "./node/num/numcomponent";
import { AddComponent } from "./node/add/addcomponent";
import { StringComponent } from "./node/string/StringComponent";
import { ServerComponent } from "./node/server/ServerComponent";
import { ListComponent } from "./node/list/ListComponent";
import { InstanceTypeComponent } from "./node/InstanceType/InstanceTypeComponent";

export default {
  list: [
    new NumComponent(),
    new AddComponent(),
    new ServerComponent(),
    new StringComponent(),
    new InstanceTypeComponent(),
    new ListComponent(),
  ],
  get(name: string) {
    const comp = this.list.find(
      item => item.name.toUpperCase() === name.toUpperCase(),
    );

    if (!comp) throw new Error(`Rete component '${name}' not found`);
    return comp;
  },
};
