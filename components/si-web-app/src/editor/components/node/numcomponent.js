import Rete from "rete";
import NumControl from "./numcontrol"
import sockets from '../../sockets';

export default class NumComponent extends Rete.Component {
  constructor() {
    super("Number");
    console.log("NumComponent.constructor() with name: " + this.name)
  }

  builder(node) {
    console.log("NumComponent.builder()")
    var out1 = new Rete.Output('num', "Number", sockets.num);
    return node.addControl(new NumControl(this.editor, 'num')).addOutput(out1);
  }

  worker(node, inputs, outputs) {
    console.log("NumComponent.worker()")
    outputs['num'] = node.data.num;
  }
}
