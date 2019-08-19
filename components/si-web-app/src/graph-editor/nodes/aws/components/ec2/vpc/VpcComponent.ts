import Rete from "rete";
import { VpcControl } from "./VpcControl";
import sockets from "../../../../sockets";

export class VpcComponent extends Rete.Component {
  constructor() {
    super("ec2-vpc");
  }

  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  builder(node) {
    var out1 = new Rete.Output("vpc", "out", sockets.num);
    return node.addControl(new VpcControl(this.editor, "vpc")).addOutput(out1);
  }

  // @ts-ignore: Parameters 'node', 'inputs', and 'outputs' implicitly have an 'any' type.
  worker(node, inputs, outputs) {}
}
