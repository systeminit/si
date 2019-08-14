import Rete from "rete";
import { InstanceTypeControl } from "./InstanceTypeControl";
import sockets from "../../../../sockets";

export class InstanceTypeComponent extends Rete.Component {
  constructor() {
    super("ec2-instance-type");
  }

  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  builder(node) {
    var out1 = new Rete.Output("instance-type", "out", sockets.num);
    return node
      .addControl(new InstanceTypeControl(this.editor, "str"))
      .addOutput(out1);
  }

  // @ts-ignore: Parameters 'node', 'inputs', and 'outputs' implicitly have an 'any' type.
  worker(node, inputs, outputs) {}
}
