import Rete from "rete";
import sockets from "../../../../sockets";

export class SecurityGroupComponent extends Rete.Component {
  constructor() {
    super("ec2-security-group");
  }
  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  builder(node) {
    let input1 = new Rete.Input("name", "name", sockets.num);
    let input2 = new Rete.Input("vpc", "vpc", sockets.num);
    let input3 = new Rete.Input("rules", "rules", sockets.num);

    var output = new Rete.Output("security-group", "out", sockets.num);

    return node
      .addInput(input1)
      .addInput(input2)
      .addInput(input3)

      .addOutput(output);
  }

  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  worker(node, inputs, outputs) {}
}
