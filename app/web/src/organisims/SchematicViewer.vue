<template>
  <div :id="viewer.id" ref="viewer" class="w-full h-full">
    <Viewer
      :schematic-viewer-id="viewer.id"
      :scene-graph-data="sceneGraphData"
      :viewer-state="state"
    />
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import _ from "lodash";

import Viewer from "./SchematicViewer/Viewer.vue";
import { SceneData } from "./SchematicViewer/Viewer/scene";
import { NodeData } from "./SchematicViewer/Viewer/geo";

import { ViewerStateMachine } from "./SchematicViewer/state";

export interface Data {
  component: {
    id: string;
  };
  viewer: {
    id: string;
    element: HTMLElement | null;
  };
  state: ViewerStateMachine;
}

export default defineComponent({
  name: "SchematicViewer",
  components: {
    Viewer,
  },
  data(): Data {
    const id = _.uniqueId();
    const viewerId = this.$options.name + "-" + id;
    const viewerState = new ViewerStateMachine();
    return {
      component: {
        id: id,
      },
      viewer: {
        id: viewerId,
        element: null,
      },
      state: viewerState,
    };
  },
  computed: {
    sceneGraphData(): SceneData {
      const nodeA: NodeData = {
        name: "node01",
        position: {
          x: 100,
          y: 100,
        },
      };

      const nodeB: NodeData = {
        name: "node02",
        position: {
          x: 300,
          y: 100,
        },
      };

      let sceneGraph: SceneData = {
        nodes: [nodeA, nodeB],
      };
      return sceneGraph;
    },
  },
  mounted(): void {
    this.viewer.element = this.$refs.viewport as HTMLElement;
  },
});
</script>
