import Rete from "rete";

import NumComponent from './node/numcomponent';
import AddComponent from './node/addcomponent';

export default {
  list : [
    new NumComponent(),
    new AddComponent(),
    ],
  get(name) {
    const comp = this
        .list
        .find(item => item.name.toUpperCase() === name.toUpperCase());

    if (!comp) 
        throw new Error(`Rete component '${name}' not found`);
    return comp;
  }
};
