import Rete from "rete";
import sockets from "../../../sockets";

export class ListComponent extends Rete.Component {
  editor: any;

  constructor() {
    super("List");
  }
  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  builder(node) {
    var inp1 = new Rete.Input("list", "List", sockets.num);
    var inp2 = new Rete.Input("list2", "List2", sockets.num);
    var out = new Rete.Output("list", "List", sockets.num);

    return node
      .addInput(inp1)
      .addInput(inp2)
      .addOutput(out);
  }

  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  worker(node, inputs, outputs) {}
}
