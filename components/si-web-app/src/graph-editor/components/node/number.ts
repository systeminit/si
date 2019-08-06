import Rete from "rete";
import sockets from "../../sockets";

export default class Number extends Rete.Component {
  constructor() {
    super("Number");
  }

  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  builder(node) {
    let out = new Rete.Output("num", "Number", sockets.num);
    return node.addOutput(out);
  }

  // @ts-ignore: Parameters 'node', 'inputs', and 'outputs' implicitly have an 'any' type.
  worker(node, inputs, outputs) {
    outputs["num"] = node.data.num;
  }
}
