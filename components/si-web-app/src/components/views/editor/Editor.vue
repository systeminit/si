<template>
  <div
    ref="editor"
    id="editor"
    class="flex flex-row h-full w-full"
    v-on:mousemove="mouseMove"
    v-on:mousedown="mouseDown"
    v-on:mouseup="mouseUp"
  >
    <div
      ref="leftPanel"
      class="bg-gray-900 w-3/5"
      :class="leftPanelVisibilityClasses"
    >
      <SchematicPanel @maximizePanelMsg="maximizePanel" />
    </div>

    <div
      ref="resizeHandle"
      class="bg-gray-800 cursor-resize"
      :class="resizeHandleVisibilityClasses"
    />

    <div
      ref="rightPanel"
      class="bg-gray-900 w-2/5"
      :class="rightPanelVisibilityClasses"
    >
      <PropertyPanel @maximizePanelMsg="maximizePanel" />
    </div>
  </div>
</template>

<script>
import { registry } from "si-registry";

import SchematicPanel from "./EditorSchematicPanel";
import PropertyPanel from "./EditorPropertyPanel";

export default {
  name: "Editor",
  components: {
    SchematicPanel,
    PropertyPanel,
  },
  data: function() {
    return {
      resizeHandle: {},
      leftPanel: {},
      rightPanel: {},
      isResizing: false,
      window: {
        width: 0,
        height: 0,
      },
      panel: {
        schematic: {
          isVisible: true,
        },
        resizeHandle: {
          isVisible: true,
        },
        property: {
          isVisible: true,
        },
      },
      msgSchematicPanel: "",
    };
  },
  mounted: async function() {
    this.resizeHandle = this.$refs.resizeHandle;
    this.leftPanel = this.$refs.leftPanel;
    this.rightPanel = this.$refs.rightPanel;
    const workspace = this.$store.getters["user/currentWorkspace"];
  },
  created() {
    window.addEventListener("resize", this.handleResize);
    this.handleResize();
  },
  destroyed() {
    window.removeEventListener("resize", this.handleResize);
  },
  methods: {
    mouseDown() {
      if (event.target === this.resizeHandle) {
        this.isResizing = true;
      }
    },
    mouseMove(event) {
      if (this.isResizing) {
        let pointerXposRelative = event.clientX - this.resizeHandle.offsetLeft;
        let panelRect = this.leftPanel.getBoundingClientRect();
        let panelWidthUpdate = parseInt(panelRect.width) + pointerXposRelative;

        this.leftPanel.style.width = panelWidthUpdate + "px";
        this.leftPanel.style.flexGrow = 0;
      }
    },
    mouseUp() {
      this.isResizing = false;
    },
    handleResize() {
      // Will need to implement left panel resize to maintain proportion when resizing the browser.
      this.window.width = window.innerWidth;
      this.window.height = window.innerHeight;
    },
    maximizePanel(msg) {
      // console.log("event")
      switch (msg.panel.id) {
        case "property":
          this.togglePanelVisibility("schematic");
          break;

        case "schematic":
          this.togglePanelVisibility("property");
          break;
      }
    },
    togglePanelVisibility: function(panelName) {
      this.panel[panelName].isVisible = !this.panel[panelName].isVisible;
      this.panel.resizeHandle.isVisible = !this.panel.resizeHandle.isVisible;
    },
    nodeList() {
      const workspace = this.$store.getters["user/currentWorkspace"];
      return this.$store.getters.entity.forWorkspace(workspace["id"]);
    },
  },
  computed: {
    leftPanelVisibilityClasses: function() {
      return {
        "panel-is-hidden": !this.panel["schematic"].isVisible,
        "panel-is-visible": this.panel["schematic"].isVisible,
      };
    },
    resizeHandleVisibilityClasses: function() {
      return {
        "resize-handle-is-hidden": !this.panel["resizeHandle"].isVisible,
        "resize-handle-is-visible": this.panel["resizeHandle"].isVisible,
      };
    },
    rightPanelVisibilityClasses: function() {
      return {
        "panel-is-hidden": !this.panel["property"].isVisible,
        "panel-is-visible": this.panel["property"].isVisible,
      };
    },
  },
  //apollo: {
  //  kubernetesDeploymentEntityList: {
  //    query() {
  //      let result = registry
  //        .get("kubernetesDeploymentEntity")
  //        .graphql.query({ methodName: "list" });
  //      return result;
  //    },
  //    fetchPolicy: "no-cache",
  //    variables() {
  //      return {
  //        pageSize: "1000",
  //      };
  //    },
  //    result({ data, loading, networkStatus }) {
  //      data.kubernetesDeploymentEntityList.items.forEach(item => {
  //        let payload = {
  //          id: item.id,
  //          name: item.name,
  //          isEntity: true,
  //        };
  //        this.$store.dispatch("editor/addNode", payload);
  //      });
  //    },
  //    update(data) {
  //      console.log("apollo update!");
  //      // The returned value will update
  //      // the vue property 'pingMessage'
  //      // return data.ping
  //    },
  //  },
  //},
};
</script>

<style scoped>
.panel-is-hidden {
  @apply overflow-hidden hidden;
}

.panel-is-visible {
  @apply flex-auto overflow-auto;
}

/*.panel-property-is-visible {
  @apply flex-auto flex-grow
}*/

.resize-handle-is-visible {
  @apply w-1 flex-none;
}
.resize-handle-is-hidden {
  @apply overflow-hidden hidden;
}

/*.resize-handle-is-hidden {
  @apply overflow-hidden order-last w-0;
}*/
</style>
