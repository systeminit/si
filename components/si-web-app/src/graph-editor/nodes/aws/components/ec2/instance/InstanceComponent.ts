import Rete from "rete";
import sockets from "../../../../sockets";

export class InstanceComponent extends Rete.Component {
  constructor() {
    super("ec2-instance");
  }
  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  builder(node) {
    let input1 = new Rete.Input("name", "name", sockets.num);
    let input2 = new Rete.Input("instanceType", "instanceType", sockets.num);
    let input3 = new Rete.Input(
      "securityGroups",
      "securityGroups",
      sockets.num,
    );
    let input4 = new Rete.Input("ami", "ami", sockets.num);
    let input5 = new Rete.Input("keyName", "keyName", sockets.num);
    let input6 = new Rete.Input("tags", "tags", sockets.num);

    var output = new Rete.Output("instance", "out", sockets.num);

    return node
      .addInput(input1)
      .addInput(input2)
      .addInput(input3)
      .addInput(input4)
      .addInput(input5)
      .addInput(input6)

      .addOutput(output);
  }

  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  worker(node, inputs, outputs) {}
}
