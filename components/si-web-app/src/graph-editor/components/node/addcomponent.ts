import Rete from "rete";
import NumControl from "./numcontrol";
import sockets from "../../sockets";

export default class AddComponent extends Rete.Component {
  editor: any;

  constructor() {
    super("Add");
  }
  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  builder(node) {
    var inp1 = new Rete.Input("num", "Number", sockets.num);
    var inp2 = new Rete.Input("num2", "Number2", sockets.num);
    var out = new Rete.Output("num", "Number", sockets.num);

    inp1.addControl(new NumControl(this.editor, "num"));
    inp2.addControl(new NumControl(this.editor, "num2"));

    return node
      .addInput(inp1)
      .addInput(inp2)
      .addControl(new NumControl(this.editor, "preview", true))
      .addOutput(out);
  }

  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  worker(node, inputs, outputs) {
    var n1 = inputs["num"].length ? inputs["num"][0] : node.data.num1;
    var n2 = inputs["num2"].length ? inputs["num2"][0] : node.data.num2;
    var sum = n1 + n2;

    this.editor.nodes
      // @ts-ignore: Parameter 'n' implicitly has an 'any' type.
      .find(n => n.id == node.id)
      .controls.get("preview")
      .setValue(sum);
    outputs["num"] = sum;
  }
}
