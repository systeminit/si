import { Stage } from "konva/types/Stage";
import { Layer } from "konva/types/Layer";

interface KonvaLayer extends Vue {
  getNode(): Layer;
}

interface KonvaStage extends Vue {
  getStage(): Stage;
}

interface KonvaTransformer extends Vue {
  getNode(): Transformer;
}

declare module "@vue/runtime-core" {
  export interface GlobalComponents {
    VStage: Vue;
    VLayer: Vue;
  }
}
