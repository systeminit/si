import Rete from "rete";
import sockets from "../../sockets";

export default class Number extends Rete.Component {
  constructor() {
    super("Number");
  }

  builder(node) {
    let out = new Rete.Output("num", "Number", sockets.num);
    return node.addOutput(out);
  }

  worker(node, inputs, outputs) {
    outputs["num"] = node.data.num;
  }
}