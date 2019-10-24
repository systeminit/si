import Rete from "rete";
import { NetworkingProtocolControl } from "./NetworkingProtocolControl";
import sockets from "../../../../sockets";

export class NetworkingProtocolComponent extends Rete.Component {
  constructor() {
    super("ec2-networking-protocol");
  }

  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  builder(node) {
    var out1 = new Rete.Output("networking-protocol", "out", sockets.num);
    return node
      .addControl(new NetworkingProtocolControl(this.editor, "protocol"))
      .addOutput(out1);
  }

  // @ts-ignore: Parameters 'node', 'inputs', and 'outputs' implicitly have an 'any' type.
  worker(node, inputs, outputs) {}
}
