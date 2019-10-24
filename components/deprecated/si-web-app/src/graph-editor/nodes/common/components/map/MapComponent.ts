import Rete from "rete";
import { StringControl } from "./MapControl";

import sockets from "../../../sockets";

export class MapComponent extends Rete.Component {
  constructor() {
    super("common-map");
  }

  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  builder(node) {
    var out1 = new Rete.Output("map", "out", sockets.num);
    return node
      .addControl(new StringControl(this.editor, "map"))
      .addOutput(out1);
  }

  // @ts-ignore: Parameters 'node', 'inputs', and 'outputs' implicitly have an 'any' type.
  worker(node, inputs, outputs) {}
}
