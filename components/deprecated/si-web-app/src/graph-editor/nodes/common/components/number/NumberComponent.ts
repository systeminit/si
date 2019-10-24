import Rete from "rete";
import { NumberControl } from "./NumberControl";
import sockets from "../../../sockets";

export class NumberComponent extends Rete.Component {
  constructor() {
    super("common-number");
  }

  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  builder(node) {
    var out1 = new Rete.Output("number", "number", sockets.num);
    return node
      .addControl(new NumberControl(this.editor, "number"))
      .addOutput(out1);
  }

  // @ts-ignore: Parameters 'node', 'inputs', and 'outputs' implicitly have an 'any' type.
  worker(node, inputs, outputs) {}
}
