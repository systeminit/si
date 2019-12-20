import Rete from "rete";
import { VpcControl } from "./KeyPairControl";
import sockets from "../../../../sockets";

export class KeyPairComponent extends Rete.Component {
  constructor() {
    super("ec2-keypair");
  }

  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  builder(node) {
    var out1 = new Rete.Output("ami", "out", sockets.num);
    return node.addControl(new VpcControl(this.editor, "ami")).addOutput(out1);
  }

  // @ts-ignore: Parameters 'node', 'inputs', and 'outputs' implicitly have an 'any' type.
  worker(node, inputs, outputs) {}
}
