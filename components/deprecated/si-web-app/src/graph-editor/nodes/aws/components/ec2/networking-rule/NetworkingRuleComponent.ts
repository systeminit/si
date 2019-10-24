import Rete from "rete";
import sockets from "../../../../sockets";

import { NetworkingRuleControl } from "./NetworkingRuleControl";

export class NetworkingRuleComponent extends Rete.Component {
  constructor() {
    super("ec2-networking-rule");
  }
  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  builder(node) {
    let input1 = new Rete.Input("name", "name", sockets.num);
    let input2 = new Rete.Input("protocol", "protocol", sockets.num);
    let input3 = new Rete.Input("fromPort", "fromPort", sockets.num);
    let input4 = new Rete.Input("toPort", "toPort", sockets.num);
    let input5 = new Rete.Input("cidrBlocks", "cidrBlocks", sockets.num);

    var output = new Rete.Output("networking-rule", "out", sockets.num);

    return node
      .addInput(input1)
      .addInput(input2)
      .addInput(input3)
      .addInput(input4)
      .addInput(input5)
      .addControl(new NetworkingRuleControl(this.editor, "cidr"))
      .addOutput(output);
  }

  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  worker(node, inputs, outputs) {}
}
