import Rete from "rete";
import VueNumControl from './NumControl.vue';
import sockets from '../../sockets';

export default class NumControl extends Rete.Control {
  constructor(emitter, key, readonly) {
    super(key);
    this.component = VueNumControl;
    this.props = { emitter, ikey: key, readonly };
  }

  setValue(val) {
    this.vueContext.value = val;
  }
}