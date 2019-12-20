import { StringComponent } from "./components/string/StringComponent";
import { ListComponent } from "./components/list/ListComponent";
import { MapComponent } from "./components/map/MapComponent";
import { NumberComponent } from "./components/number/NumberComponent";

export default {
  list: [
    new StringComponent(),
    new ListComponent(),
    new MapComponent(),
    new NumberComponent(),
  ],
  get(name: string) {
    const comp = this.list.find(
      item => item.name.toUpperCase() === name.toUpperCase(),
    );

    if (!comp) throw new Error(`Rete component '${name}' not found`);
    return comp;
  },
};
