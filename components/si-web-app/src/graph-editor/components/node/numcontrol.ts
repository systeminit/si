import Rete from "rete";
import VueNumControl from "./VueNumControl.vue";
import sockets from "../../sockets";

export default class NumControl extends Rete.Control {
  component: any; // Fix this to the right type.
  props: any; // Fix this to the right type.
  vueContext: any;

  // @ts-ignore: Parameter 'emitter' and 'key' implicitly have an 'any' type.
  constructor(emitter, key, readonly?: any) {
    super(key);
    this.component = VueNumControl;
    this.props = { emitter, ikey: key, readonly };
  }

  // @ts-ignore: Parameter 'val' implicitly has an 'any' type.
  setValue(val) {
    this.vueContext.value = val;
  }
}
