import Rete from "rete";
import { NumControl } from "./numcontrol";
import sockets from "../../../sockets";

export class NumComponent extends Rete.Component {
  constructor() {
    super("Number");
  }

  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  builder(node) {
    var out1 = new Rete.Output("num", "Number", sockets.num);
    return node.addControl(new NumControl(this.editor, "num")).addOutput(out1);
  }

  // @ts-ignore: Parameters 'node', 'inputs', and 'outputs' implicitly have an 'any' type.
  worker(node, inputs, outputs) {
    outputs["num"] = node.data.num;
  }
}
