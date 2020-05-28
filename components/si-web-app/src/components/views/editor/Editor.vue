<template>
  <div
    ref="editor"
    id="editor"
    class="flex flex-row h-full w-full"
    v-on:mousemove="mouseMove"
    v-on:mousedown="mouseDown"
    v-on:mouseup="mouseUp"
  >
<!--     <div ref="leftPanel" class="box-border flex-auto bg-gray-900">
      <SchematicPanel />
    </div>

    <div ref="resizeHandle" class="w-1 bg-gray-800 flex-none cursor-resize" />
 -->
    <div ref="rightPanel" class="box-border flex-auto bg-gray-900 h-full">
      <PropertyPanel />
    </div>
  </div>
</template>

<script>
import SchematicPanel from "./EditorSchematicPanel.vue";
import PropertyPanel from "./EditorPropertyPanel";

export default {
  name: "Editor",
  components: {
    // SchematicPanel,
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
      // console.log(this.window.width)
      // console.log(this.window.height)

      // get size ratio between left and right side panel
      // transform width of the right pannel while maintaining ratios
      // set right panel width while resizing window.
      // simulate resizeHandle transform...
    },
  },
};
</script>
