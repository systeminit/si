<template>
  <div id="schematic-panel" class="w-full h-full grid-background">
    <div
      ref="canvasParent"
      class="w-full h-full overflow-hidden"
      @wheel="mouseWheel"
      @mousedown="mouseDown"
      @mousemove="mouseMove"
      @mouseup="mouseUp"
    >
      <div
        ref="canvas"
        id="canvas"
        class="relative flex-auto block w-full h-full canvas"
      >
        <ConnectionObject
          class="connections"
          v-for="(connection, index) in connectionList"
          v-bind:key="index"
          :sourceNodePosition="connection.sourceNodePosition"
          :destinationNodePosition="connection.destinationNodePosition"
        />

        <NodeList />

        <svg
          ref="grid"
          id="node-grid"
          :width="gridWidth"
          :height="gridHeight"
          xmlns="http://www.w3.org/2000/svg"
        >
          <defs>
            <pattern
              id="grid"
              width="20"
              height="20"
              patternUnits="userSpaceOnUse"
              class="grid-background"
            >
              <rect fill="#1E1E1E" width="20" height="20" />
              <path
                d="M 20 0 L 0 0 0 20"
                fill="none"
                stroke="#323537"
                stroke-width="1"
              />
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#grid)" />
        </svg>
      </div>
    </div>
  </div>
</template>

<script>
import Vue from "vue";
import NodeList from "./NodeList.vue";
import ConnectionObject from "./ConnectionObject.vue";

import { mapGetters, Store } from "vuex";
import { debouncedSetPosition } from "@/store/modules/node";
import _ from "lodash";

export default {
  name: "SchematicPanel",
  components: {
    NodeList,
    ConnectionObject,
  },
  computed: {
    ...mapGetters({
      edgeConnectionList: "edge/connectionList",
      nodeList: "node/list",
      nodeSelection: "node/current",
      mouseTrackSelection: "node/mouseTrackSelection",
    }),
    connectionList() {
      const positions = [];
      for (const edge of this.edgeConnectionList) {
        let sourceNodePosition;
        let destinationNodePosition;
        if (this.nodeListPositions[edge.tailVertex.id]) {
          sourceNodePosition = this.nodeListPositions[edge.tailVertex.id];
        } else {
          let sourceNode = _.find(this.nodeList, ["id", edge.tailVertex.id]);
          if (sourceNode) {
            sourceNodePosition = sourceNode.position;
          } else {
            continue;
          }
        }
        if (this.nodeListPositions[edge.headVertex.id]) {
          destinationNodePosition = this.nodeListPositions[edge.headVertex.id];
        } else {
          let destinationNode = _.find(this.nodeList, [
            "id",
            edge.headVertex.id,
          ]);
          if (destinationNode) {
            destinationNodePosition = destinationNode.position;
          } else {
            continue;
          }
        }

        positions.push({
          sourceNodePosition,
          destinationNodePosition,
        });
      }
      return positions;
    },
  },
  watch: {
    edgeConnectionList(_new, _old) {
      this.nodeListPositions = {};
    },
  },
  data() {
    return {
      selectedNode: "",
      gridWidth: "100%",
      gridHeight: "100%",
      isPanning: false,
      isDragging: false,
      isZooming: false,
      mouse: {
        position: {
          screen: {
            x: 0,
            y: 0,
          },
          element: {
            x: 0,
            y: 0,
          },
        },
      },
      pan: {
        translation: {
          x: 0,
          y: 0,
        },
        mouse: {
          x: 0,
          y: 0,
        },
      },
      zoom: {
        sensitivity: 0.001,
        center: {
          x: 0,
          y: 0,
        },
        translation: {
          x: 0,
          y: 0,
        },
        mouse: {
          x: 0,
          y: 0,
        },
        factor: 1,
        // min: 0.25,
        min: 1,
        max: 1,
      },
      canvas: {
        element: null,
        translation: {
          x: 0,
          y: 0,
        },
        offset: {
          x: 0,
          y: 0,
        },
      },
      grid: {
        element: null,
        dimension: {
          width: 1,
          height: 1,
        },
      },
      editor: {
        element: null,
      },
      selection: {
        object: null,
        objectPosition: {
          x: 0,
          y: 0,
        },
        element: null,
        origin: {
          x: 0,
          y: 0,
        },
        offset: {
          x: 0,
          y: 0,
        },
        translation: {
          x: 0,
          y: 0,
        },
      },
      nodeListPositions: {},
    };
  },
  mounted: function() {
    this.canvas.element = this.$refs.canvas;
    this.grid.element = this.$refs.grid;

    /**
     * TODO
     *
     * the next line is dangerous and it sucks!
     * (this.editor.element = this.$parent.$parent.$refs.editor)
     * -> need to find a better way to do this
     */
    this.editor.element = this.$parent.$parent.$refs.editor;

    // Position grid
    let canvasOrigin = {
      x: this.canvas.element.offsetLeft,
      y: this.canvas.element.offsetTop,
    };
    // console.log(canvasOrigin);

    let canvasDimension = {
      width: this.canvas.element.offsetWidth,
      height: this.canvas.element.offsetHeight,
    };
    // console.log(canvasDimension);

    let viewCenter = {
      x: canvasDimension.width / 2 - canvasOrigin.x,
      y: canvasDimension.height / 2 - canvasOrigin.y,
    };
    // console.log(viewCenter);

    let gridDimension = {
      x: this.grid.element.width.baseVal.value,
      y: this.grid.element.height.baseVal.value,
    };

    // console.log(gridDimension);

    // Initial grid size!
    // let scaleFactor = 4;
    let scaleFactor = 1;

    let gridUpdatedDimension = {
      x: 100 * scaleFactor,
      y: 100 * scaleFactor,
    };
    this.gridWidth = `${gridUpdatedDimension.x}%`;
    this.gridHeight = `${gridUpdatedDimension.y}%`;
    // console.log(gridUpdatedDimension);

    this.grid.dimension.width = gridDimension.x * scaleFactor;
    this.grid.dimension.height = gridDimension.y * scaleFactor;
    // console.log(this.grid.dimension.width, this.grid.dimension.height);

    gridDimension = {
      x: this.grid.element.width.baseVal.value * scaleFactor,
      y: this.grid.element.height.baseVal.value * scaleFactor,
    };

    let gridCenter = {
      x: gridDimension.x / 2,
      y: gridDimension.y / 2,
    };
    // console.log(gridCenter);

    let canvasOffset = {
      x: -(gridCenter.x - viewCenter.x - canvasOrigin.x),
      y: -(gridCenter.y - viewCenter.y - canvasOrigin.y),
    };
    this.canvas.element.style.transform = `translate(${canvasOffset.x}px, ${canvasOffset.y}px)`;

    this.zoom.translation.x = canvasOffset.x;
    this.zoom.translation.y = canvasOffset.y;
    this.canvas.offset.x = canvasOffset.x;
    this.canvas.offset.y = canvasOffset.y;
  },
  methods: {
    updateSelectedNodePosition(posX, posY) {
      let nodePosition = {
        x: posX,
        y: posY,
      };
      Vue.set(this.nodeListPositions, this.nodeSelection.id, nodePosition);
      debouncedSetPosition({
        store: this.$store,
        nodeId: this.nodeSelection.id,
        nodePosition,
      });
    },
    mouseOver() {},
    mouseDown(event) {
      // console.log("mouseDown");
      // Enable alt key to drag the whole editor canvas
      // if (event.altKey) {
      //   let mousePositionInScreenSpace = {
      //     x: event.clientX,
      //     y: event.clientY,
      //   };
      //   this.log(
      //     `mousePositionInScreenSpace - x:${mousePositionInScreenSpace.x} y:${mousePositionInScreenSpace.y}`,
      //   );

      //   this.mouse.position.screen.x = mousePositionInScreenSpace.x;
      //   this.mouse.position.screen.y = mousePositionInScreenSpace.y;

      //   this.isPanning = true;
      // } else {
      //   this.selection.object = event.target;
      //   // console.log(this.selection.object);

      // console.log(this.nodeSelection)
      // if (this.nodeSelection.id) {
      //   this.selection.object = document.getElementById(this.nodeSelection.id)
      // } else {
      //   this.selection.object = event.target;
      // }

      if (event.target.classList.contains("node")) {
        if (this.nodeSelection.id) {
          this.selection.object = document.getElementById(
            this.nodeSelection.id,
          );
        } else {
          this.selection.object = null;
        }
      } else {
        this.selection.object = null;
      }

      if (this.selection.object) {
        // console.log(this.selection.object);
        // console.log(this.selection.object.$root);
        // console.log(event);

        let selectionOffsetLeft = this.selection.object.offsetLeft;
        let selectionOffsetTop = this.selection.object.offsetTop;

        let mousePositionX = event.clientX;
        let mousePositionY = event.clientY;

        this.selection.offset.x = mousePositionX - selectionOffsetLeft;
        this.selection.offset.y = mousePositionY - selectionOffsetTop;
      }
    },
    mouseUp() {
      if (this.mouseTrackSelection) {
        this.$store.dispatch("node/unsetMouseTrackSelection");
      }

      this.selection.object = null;

      if (this.isPanning == true) {
        // So that we remember where the canvas is next time we pan
        this.canvas.offset.x = this.canvas.translation.x;
        this.canvas.offset.y = this.canvas.translation.y;

        // So that the zoom know where the canvas is.
        this.zoom.translation.x = this.canvas.offset.x;
        this.zoom.translation.y = this.canvas.offset.y;

        this.isPanning = false;
      }
    },
    mouseMove(event) {
      if (this.mouseTrackSelection) {
        this.selection.object = document.getElementById(
          this.mouseTrackSelection,
        );
      }

      const selectedObject = this.selection.object;

      if (this.isPanning == true) {
        this.panCanvas();
      } else {
        if (selectedObject !== null) {
          if (selectedObject.classList.contains("node")) {
            let mousePositionX = event.clientX;
            let mousePositionY = event.clientY;

            // Need to account for zoom factor? (1 / this.zoom.factor)
            let newPositionX = mousePositionX - this.selection.offset.x;
            let newPositionY = mousePositionY - this.selection.offset.y;

            // We can refactor this to work via the node component instead of this here.
            // need to test performance...
            selectedObject.style.left = newPositionX + "px";
            selectedObject.style.top = newPositionY + "px";

            this.updateSelectedNodePosition(newPositionX, newPositionY);
          }
        }
      }
    },
    mouseWheel(event) {
      if (event.altKey) {
        this.zoomCanvas(event);
      }
    },
    log(msg) {
      console.log(msg);
    },
    canvasConstraint() {
      let left = this.canvas.element.offsetRight;
      let top = this.canvas.element.offsetBottom;

      // console.log(left, top);

      // console.log(
      //   "::" + this.grid.element.getBoundingClientRect().left,
      //   this.grid.element.getBoundingClientRect().top,
      // );
      // console.log(
      //   "::" + this.canvas.element.offsetLeft,
      //   this.canvas.element.offsetTop,
      // );
    },
    panCanvas() {
      // console.log("==============");
      // console.log("panning canvas");

      let mousePositionInScreenSpace = {
        x: event.clientX,
        y: event.clientY,
      };
      this.log(
        `mousePositionInScreenSpace - x:${mousePositionInScreenSpace.x} y:${mousePositionInScreenSpace.y}`,
      );

      let mouseMovement = {
        x: mousePositionInScreenSpace.x - this.mouse.position.screen.x,
        y: mousePositionInScreenSpace.y - this.mouse.position.screen.y,
      };
      this.log(`mouseMovement - x:${mouseMovement.x} y:${mouseMovement.y}`);

      let translation = {
        x: this.canvas.offset.x + mouseMovement.x,
        y: this.canvas.offset.y + mouseMovement.y,
      };

      // console.log(
      //   `canvas offset - x:${this.canvas.offset.x} y:${this.canvas.offset.y}`,
      // );
      this.log(
        `desired adjusted translation - x:${translation.x} y:${translation.y}`,
      );
      this.log(
        `translation limits - x:0 to ${1 -
          this.editor.element.offsetWidth} y:0 to ${1 -
          this.editor.element.offsetHeight}`,
      );

      // Restrict translations to the view
      translation.x = Math.min(0, translation.x);
      translation.x = Math.max(
        1 - this.editor.element.offsetWidth,
        translation.x,
      );

      translation.y = Math.min(0, translation.y);
      translation.y = Math.max(
        1 - this.editor.element.offsetHeight,
        translation.y,
      );

      // X axis
      let pt1 = {
        x: 1,
        y: 1 - 1135,
      };

      let pt2 = {
        x: 4,
        y: 0,
      };

      // Y axis
      let pt3 = {
        x: 1,
        y: 1 - 550,
      };

      let val = 1 / this.zoom.factor;

      let limitX = this.linearEquation(pt1, pt2, val);
      let limitY = this.linearEquation(pt3, pt2, val);

      // console.log(limitX);

      translation.x = Math.max(limitX, translation.x);
      // translation.y = Math.max(limitY, translation.y)
      // console.log(limitY);

      this.log(`adjusted translation - x:${translation.x} y:${translation.y}`);
      // console.log("zoom " + 1 / this.zoom.factor);

      this.canvas.element.style.transformOrigin = "0 0";
      this.canvas.element.style.transform = `matrix(${this.zoom.factor}, 0, 0, ${this.zoom.factor}, ${translation.x}, ${translation.y})`;

      this.canvas.translation.x = translation.x;
      this.canvas.translation.y = translation.y;
    },
    linearEquation(point1, point2, value) {
      /**
       * - solving the equation to limit pan -
       *
       * when scaleFactor is 1 we want this.editor.element.offsetWidth
       * when scaleFactor is 0.25 we want 0
       *
       * refresher
       * line equation { y = mx+b }
       * m: slope
       * b: y intercept
       *
       */

      // console.log(point1);/
      // console.log(point2);

      // Solving the slope
      let m = (point2.y - point1.y) / (point2.x - point1.x);
      // console.log(`m: ${m}`);

      // solving for b (with any of the two point)
      let b = point1.y - m * point1.x;
      // console.log(`b: ${b}`);

      // solve for y
      let y = m * value + b;
      // console.log(`y: ${y}`);

      return y;
    },
    zoomCanvas(event) {
      /**
       * - Zoom on cursor implementation -
       *
       * This could be written in much fewer lines. Since not everyone is
       * comfortable with 2d transforms, I have opted for readability.
       *
       * This implementation uses css transform matrix to scales and
       * translate the <div> element in order to simulate zooming in at a
       * specific location.
       *
       * 1: calculate zoom scale
       * 2: calculate zoom translation
       * 3: perform transforms
       */

      // prevent mouse wheel default behavior
      event.preventDefault();

      /**
       * @mouseScrollAmount
       * 1: get vertical scroll amount from the mouse event
       * 2: apply zoom sensitivity multiplier (scroll intensity)
       *
       * @event.deltaY :: mouse vertical scroll
       * @this.zoom.sensitivity :: our sensitivity setting
       */
      let mouseScrollAmount = event.deltaY * this.zoom.sensitivity;

      /**
       * @zoomFactor :: new zoom factor
       * 1: add new mouse scroll amount to previous mouse scroll factor
       * 2: restrict mouse factor to our min and max range
       *
       * @this.zoom.factor represents last zoom factor
       * @this.zoom.min represents our minimum zoom setting
       * @this.zoom.max represents our maximum zoom setting
       */
      let zoomFactor = this.zoom.factor + mouseScrollAmount;
      zoomFactor = Math.min(this.zoom.max, Math.max(this.zoom.min, zoomFactor));

      /**
       * @zoomDeltaPercentage :: difference (in percentage) between
       *  previous and new zoom factor
       *
       * @this.zoom.factor :: previous zoom factor
       * @zoomFactor :: new zoom factor
       */
      let zoomDeltaPercentage = 1 - zoomFactor / this.zoom.factor;

      /**
       * @canvasOrigin :: Canvas origin
       *
       * 1: get the canvas <div> element
       * 2: get the canvas offset
       *
       * @canvas.offsetLeft :: offsetLeft position of the canvas <div> element
       * @canvas.offsetTop :: offsetTop position of the canvas <div> element
       */
      let canvas = this.$refs.canvas;
      let canvasOrigin = {
        x: canvas.offsetLeft,
        y: canvas.offsetTop,
      };

      /**
       * - Computing the canvas translation -
       *
       * Because the zoom (scale transform) is applied at the canvas origin,
       * we must translate the canvas to simulate a scale transform
       * applied at the mouse position.
       *
       * 1: get the mouse position in the canvas screen space
       * 2: transform the mouse position to canvas space
       * 3: calculate the canvas translation
       *
       */

      /**
       * @mousePositionInScreenSpace :: in canvas <div> element screen space
       *
       * @event.clientX :: mouse pointer horizontal coordinate
       * @event.clientY :: mouse pointer vertical coordinate
       */
      let mousePositionInScreenSpace = {
        x: event.clientX - canvasOrigin.x,
        y: event.clientY - canvasOrigin.y,
      };

      /**
       * @mousePositionInCanvasSpace :: in (transformed) canvas space
       *
       * @mousePositionInScreenSpace :: mouse position in screen spcae
       * @this.zoom.translation :: previous canvas translation
       * @zoomDeltaPercentage :: difference (in percentage) between
       *  previous and new zoom factor
       */
      let mousePositionInCanvasSpace = {
        x:
          (mousePositionInScreenSpace.x - this.zoom.translation.x) *
          zoomDeltaPercentage,
        y:
          (mousePositionInScreenSpace.y - this.zoom.translation.y) *
          zoomDeltaPercentage,
      };

      /**
       * @translation :: canvas translation
       *
       * @this.zoom.translation :: previous canvas translation
       * @mousePositionInCanvasSpace :: mouse position in canvas space
       */
      let translation = {
        x: this.zoom.translation.x + mousePositionInCanvasSpace.x,
        y: this.zoom.translation.y + mousePositionInCanvasSpace.y,
      };

      // Restrict translations to the view
      translation.x = Math.min(0, translation.x);
      translation.x = Math.max(
        1 - this.editor.element.offsetWidth,
        translation.x,
      );

      translation.y = Math.min(0, translation.y);
      translation.y = Math.max(
        1 - this.editor.element.offsetHeight,
        translation.y,
      );

      /**
       * - The actual zoom transform -
       *
       * 1: we set the transform origin to 0,0 (scale center)
       * 2: we apply the scale and translation using a matrix
       *
       */
      canvas.style.transformOrigin = "0 0";
      canvas.style.transform = `matrix(${zoomFactor}, 0, 0, ${zoomFactor}, ${translation.x}, ${translation.y})`;

      /**
       * Store state
       *
       * @this.zoom.factor - set to current zoom factor
       * @this.zoom.translation - set to current translation
       *
       */
      this.zoom.factor = zoomFactor;
      this.zoom.translation.x = translation.x;
      this.zoom.translation.y = translation.y;

      // this is so that pan knows where the canvas is ...
      this.canvas.offset.x = translation.x;
      this.canvas.offset.y = translation.y;
    },
    updateGrid(scale) {
      // let grid = this.$refs.grid;
      let width = {
        x: 100 + 100 * (1 + (1 - scale)),
        y: 100 + 100 * (1 + (1 - scale)),
      };
      this.gridWidth = `${width.x}%`;
      this.gridHeight = `${width.y}%`;

      // console.log(width);
    },
  },
};
</script>

<style type="text/css" scoped>
.connections {
  /*position: absolute;*/
  z-index: 2; /* Need to decide zindex ranges for the editor layers */
  left: 0;
  top: 0;
}

/* The grid is underneith everything */
.grid-background {
  z-index: 1;
  background-color: #111718;
}
</style>
