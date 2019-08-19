import Rete from "rete";
import { NodeEditor } from "rete";
import sockets from "../../../sockets";

const nameInputId = "name";
const instanceTypeInputId = "instanceType";
const securityGroupsInputId = "securityGroups";
const amiInputId = "ami";
const keyNameInputId = "keyName";
const userDataInputId = "userData";
const tagsInputId = "tags";

export class ServerComponent extends Rete.Component {
  editor!: NodeEditor;

  constructor() {
    super("Server");
  }
  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  builder(node) {
    let input1 = new Rete.Input(nameInputId, nameInputId, sockets.num);
    let input2 = new Rete.Input(
      instanceTypeInputId,
      instanceTypeInputId,
      sockets.num,
    );
    let input3 = new Rete.Input(
      securityGroupsInputId,
      securityGroupsInputId,
      sockets.num,
    );
    let input4 = new Rete.Input(keyNameInputId, keyNameInputId, sockets.num);
    let input5 = new Rete.Input(amiInputId, amiInputId, sockets.num);
    let input6 = new Rete.Input(userDataInputId, userDataInputId, sockets.num);
    let input7 = new Rete.Input(tagsInputId, tagsInputId, sockets.num);

    var out = new Rete.Output("num", "Number", sockets.num);

    // inp1.addControl(new NumControl(this.editor, "num"));
    // inp2.addControl(new NumControl(this.editor, "num2"));

    return node
      .addInput(input1)
      .addInput(input2)
      .addInput(input3)
      .addInput(input4)
      .addInput(input5)
      .addInput(input6)
      .addInput(input7)

      .addOutput(out);
  }

  // @ts-ignore: Parameter 'node' implicitly has an 'any' type.
  worker(node, inputs, outputs) {
    let n1 = inputs["str"] ? inputs["str"][0] : node.data.str1;
    let n2 = inputs["str2"] ? inputs["str2"][0] : node.data.str2;

    // @ts-ignore: Parameter 'n' implicitly has an 'any' type.
    this.editor.nodes.find(n => n.id == node.id).controls.get("preview");
    // .setValue(n1);
    outputs["str"] = n1;
  }
}
