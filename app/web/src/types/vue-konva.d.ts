import { default as VStage } from "vue-konva/components/stage";

import { Stage } from "konva/types/Stage";
import { Layer } from "konva/types/Layer";
import { Node } from "konva/types/Node";

interface KonvaLayer extends Vue {
  getNode(): Layer;
}

interface KonvaStage extends Vue {
  getStage(): Stage;
}

interface KonvaTransformer extends Vue {
  getNode(): Transformer;
}

// $refs!: {
//   stage: KonvaStage
//   transformer: KonvaTransformer
//   markLayer: KonvaLayer
// }

declare module "@vue/runtime-core" {
  export interface GlobalComponents {
    VStage: Vue;
    VLayer: Vue;
  }
}
