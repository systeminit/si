import Rete from "rete";
import sockets from "../../../sockets";

export class ListComponent extends Rete.Component {
  editor: any;

  constructor() {
    super("common-list");
  }
  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  builder(node) {
    var inp1 = new Rete.Input("item1", "item", sockets.num);
    var inp2 = new Rete.Input("item2", "item", sockets.num);
    var inp3 = new Rete.Input("item3", "item", sockets.num);
    var out = new Rete.Output("list", "out", sockets.num);

    return node
      .addInput(inp1)
      .addInput(inp2)
      .addInput(inp3)
      .addOutput(out);
  }

  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  worker(node, inputs, outputs) {
    outputs["list"] = node.data.list;
  }
}
