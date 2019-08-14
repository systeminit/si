import Rete from "rete";
import { NetworkingCidrBlockControl } from "./NetworkingCidrBlockControl";

import sockets from "../../../../sockets";

export class NetworkingCidrBlockComponent extends Rete.Component {
  constructor() {
    super("ec2-cidr-block");
  }

  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  builder(node) {
    var out1 = new Rete.Output("cidr", "out", sockets.num);
    return node
      .addControl(new NetworkingCidrBlockControl(this.editor, "cidr"))
      .addOutput(out1);
  }

  // @ts-ignore: Parameters 'node', 'inputs', and 'outputs' implicitly have an 'any' type.
  worker(node, inputs, outputs) {}
}
