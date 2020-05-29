<template>
   <!-- eslint-disable vue/no-unused-components -->
  <div
    ref="editor"
    id="editor"
    class="flex flex-row h-full w-full"
    v-on:mousemove="mouseMove"
    v-on:mousedown="mouseDown"
    v-on:mouseup="mouseUp"
  >
    <div ref="leftPanel" class="box-border flex-auto bg-gray-900">
      <SchematicPanel/>
    </div>

    <div ref="resizeHandle" class="w-1 bg-gray-800 flex-none cursor-resize"/>

    <div ref="rightPanel" class="box-border flex-auto bg-gray-900">
      <PropertyPanel
        @maximizePanelMsg="maximizePanel"
      />
    </div>
  </div>
</template>

<script>
/* eslint-disable vue/no-unused-components */
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
        schematic:{
          isVisible: true
        }
      },
      msgSchematicPanel: ""
    };
  },
  mounted: function() {
    this.resizeHandle = this.$refs.resizeHandle;
    this.leftPanel = this.$refs.leftPanel;
    this.rightPanel = this.$refs.rightPanel;
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
      switch (msg.panel.id) {
        case "property":

        console.log("about to hide")
        this.togglePanelVisibility("schematic")
        console.log("sent hide")
          break;
      }

    },
    togglePanelVisibility: function(panelName) {
      console.log("togglePanelVisibility begin")
      this.panel[panelName].isVisible = !this.panel[panelName].isVisible;
      console.log("togglePanelVisibility done")

      console.log(this.panel["schematic"].isVisible)
    },
  },
  computed: {
    leftPanelVisibilityClasses: function() {
      console.log("computing")
      return {
        'is-hidden': !this.panel["schematic"].isVisible,
      };
    },
    resizeHandleVisibilityClasses: function() {
      console.log("computing")
      return {
        'is-hidden': !this.panel["schematic"].isVisible,
      };
    },
  }
};
</script>

<style scoped>
.is-hidden {
  @apply overflow-hidden h-0 w-0;
}


</style>
