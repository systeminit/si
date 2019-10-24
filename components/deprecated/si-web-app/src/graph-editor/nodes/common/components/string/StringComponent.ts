import Rete from "rete";
import { StringControl } from "./StringControl";

import sockets from "../../../sockets";

export class StringComponent extends Rete.Component {
  constructor() {
    super("common-string");
  }

  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  builder(node) {
    var out1 = new Rete.Output("str", "out", sockets.num);
    return node
      .addControl(new StringControl(this.editor, "str"))
      .addOutput(out1);
  }

  // @ts-ignore: Parameters 'node', 'inputs', and 'outputs' implicitly have an 'any' type.
  worker(node, inputs, outputs) {
    outputs["str"] = node.data.str;
  }
}
