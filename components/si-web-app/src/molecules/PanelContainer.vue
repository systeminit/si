<template>
  <div class="flex flex-col w-full h-full" :id="vueHolderContainerRefName()">
    <!-- panel container -->
    <div :id="panelContainerRefName()" :class="panelContainerClasses()">
      <!-- panel holder -->
      <div
        v-for="(panel, panelIndex) in panelContainer.panels"
        :key="panelIndex"
        :class="panelHolderClasses(panel)"
        :style="panelHolderStyles()"
        :id="panelHolderRefName(panelIndex)"
      >
        <!-- panel -->
        <div
          :id="panelRefName(panelIndex)"
          :class="panelClasses(panel)"
          :style="panelStyles()"
          v-if="panel.type == 'panel'"
        >
          <PanelSelector
            ref="panelSelector"
            :panelIndex="panelIndex"
            :panelRef="panelRefName(panelIndex)"
            :panelContainerRef="panelContainerRefName()"
            :initialPanelType="panel.name"
          />
        </div>
        <PanelContainer
          :panelContainer="panel"
          :index="panelIndex"
          :parentPrefix="prefix()"
          v-else
        />
        <!-- resizer -->
        <div
          :class="resizerHolderClasses()"
          style="min-width: 4px; min-height: 4px;"
          v-if="panelIndex != panelContainer.panels.length - 1"
        >
          <div
            :class="resizerClasses()"
            :style="resizerStyles()"
            @mousedown="startResize(panelIndex, $event)"
          ></div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import PanelSelector from "@/molecules/PanelSelector.vue";
import { PanelEventBus } from "@/atoms/PanelEventBus";
import _ from "lodash";

const RESIZER_SIZE = 4;
const RESIZER_HEIGHT_PERCENTAGE = 2;
const RESIZER_WIDTH_PERCENTAGE = 1;
const MIN_PANEL_COLUMN_SIZE = 50;
const MIN_PANEL_ROW_SIZE = 200;

interface IMaximized {
  panelIndex: number;
  panelRef: string;
  panelContainerRef: string;
}

interface ResizeEvent {
  panelCheech: {
    panelIndex: number;
    size: Size;
  };
  panelChong: {
    panelIndex: number;
    size: Size;
  };
  startingPosition: {
    clientX: number;
    clientY: number;
  };
}

interface Size {
  heightPx: number;
  widthPx: number;
}

interface Panel {
  name: string;
  type: "panel";
}

export type PanelOrPanelContainer = Panel | IPanelContainer;

export type IPanelContainer =
  | IPanelContainerWithoutSize
  | IPanelContainerWithSize;

export interface IPanelContainerWithoutSize {
  orientation: "column" | "row";
  panels: PanelOrPanelContainer[];
  type: "panelContainer";
  width?: never;
}

export interface IPanelContainerWithSize {
  orientation: "column" | "row";
  panels: PanelOrPanelContainer[];
  type: "panelContainer";
  width: number;
}

interface IData {
  resizeEvent: null | ResizeEvent;
  ticking: boolean;
  putBackTo: {
    parent?: HTMLElement;
    previousSibling?: HTMLElement;
  };
  maximizedData: IMaximized | null;
  originalMaximizedElementData: {
    style: string | null;
  };
  isVisible: boolean;
}

export default Vue.extend({
  name: "PanelContainer",
  components: {
    PanelSelector,
  },
  props: {
    panelContainer: Object as PropType<IPanelContainer>,
    parentPrefix: String,
    index: Number,
  },
  data(): IData {
    return {
      resizeEvent: null,
      ticking: false,
      originalMaximizedElementData: {
        style: null,
      },
      maximizedData: null,
      putBackTo: {},
      isVisible: true,
    };
  },
  mounted() {
    this.setStaticSizes();
    PanelEventBus.$on("maximize-container", this.maximizePanel);
    PanelEventBus.$on("minimize-container", this.minimizePanel);
    PanelEventBus.$on("panel-created", this.onPanelCreated);
    PanelEventBus.$on("panel-deleted", this.onPanelDeleted);
    this.setPanelIsMaximizedContainerEnabled();
    this.setInitialPanelSize();
  },
  beforeDestroy() {
    PanelEventBus.$off("maximize-container", this.maximizePanel);
    PanelEventBus.$off("minimize-container", this.minimizePanel);
    PanelEventBus.$off("panel-created", this.onPanelCreated);
    PanelEventBus.$off("panel-deleted", this.onPanelDeleted);
  },
  methods: {
    onPanelCreated(event: { panelContainerRef: string }) {
      Vue.nextTick(() => {
        if (this.panelContainerRefName() == event.panelContainerRef) {
          this.setStaticSizes();
        }
      });
    },
    onPanelDeleted() {
      Vue.nextTick(() => {
        this.setStaticSizes();
      });
    },
    prefix(): string {
      return `${this.parentPrefix}-${this.index}`;
    },
    setStaticSizes() {
      const panelContainer = this.panelContainer;
      const numberOfPanels = panelContainer.panels.length;
      const startingPercentage = Math.floor(100 / numberOfPanels);
      for (
        let panelIndex = 0;
        panelIndex < panelContainer.panels.length;
        panelIndex++
      ) {
        const panelHolderElem = document.getElementById(
          this.panelHolderRefName(panelIndex),
        );
        if (panelHolderElem) {
          if (panelHolderElem && this.panelContainer.orientation == "column") {
            panelHolderElem.setAttribute(
              "style",
              `height: ${startingPercentage}%; width: 100%;`,
            );
          } else {
            panelHolderElem.setAttribute(
              "style",
              `width: ${startingPercentage}%; height: 100%;`,
            );
          }
        }
      }
    },
    setPanelIsMaximizedContainerEnabled() {
      let panelSelectors = this.$refs.panelSelector as Vue[];
      if (panelSelectors && panelSelectors.length > 1) {
        for (let i = 0; i < panelSelectors.length; i++) {
          // @ts-ignore
          panelSelectors[i].isMaximizedContainerEnabled = true;
        }
      } else {
        if (panelSelectors && panelSelectors[0]) {
          // @ts-ignore
          panelSelectors[0].isMaximizedContainerEnabled = false;
        }
      }
    },
    setInitialPanelSize() {
      // Temporary hack to see how the layout feels.
      const panelContainer = this.panelContainer;
      const orientation = panelContainer.orientation;
      const numberOfPanels = panelContainer.panels.length;

      const panelCheechIndex = 0;
      const panelChongIndex = 1;

      const panelContainerElem = document.getElementById(
        this.panelContainerRefName(),
      );
      const panelCheechElem = this.getPanelHolderElementByIndex(
        panelCheechIndex,
      );
      const panelChongElem = this.getPanelHolderElementByIndex(panelChongIndex);

      if (orientation == "row") {
        if (panelContainerElem && panelCheechElem && panelChongElem) {
          // @ts-ignore
          if (this.panelContainer.panels[0].width) {
            // @ts-ignore
            const newCheechWidthPercent = this.panelContainer.panels[0].width;
            const newChongWidthPercent = 100 - newCheechWidthPercent;
            panelCheechElem.setAttribute(
              "style",
              `width: ${newCheechWidthPercent}%;`,
            );
            panelChongElem.setAttribute(
              "style",
              `width: ${newChongWidthPercent}%;`,
            );
          }
        }
      }
    },
    resizePanels(event: MouseEvent) {
      event.preventDefault();
      if (this.resizeEvent && !this.ticking) {
        this.ticking = true;

        const panelContainer = this.panelContainer;
        const orientation = panelContainer.orientation;
        const numberOfPanels = panelContainer.panels.length;

        const panelContainerElem = document.getElementById(
          this.panelContainerRefName(),
        );
        const panelCheechElem = this.getPanelHolderElementByIndex(
          this.resizeEvent.panelCheech.panelIndex,
        );
        const panelChongElem = this.getPanelHolderElementByIndex(
          this.resizeEvent.panelChong.panelIndex,
        );

        if (panelContainerElem && panelCheechElem && panelChongElem) {
          let containerBoundingClientRect = panelContainerElem.getBoundingClientRect();
          let panelCheechBoundingClientRect = panelCheechElem.getBoundingClientRect();
          let panelChongBoundingClientRect = panelChongElem.getBoundingClientRect();

          if (orientation == "column") {
            // Total height of the container - for figuring out percentages.
            let totalHeightPx = containerBoundingClientRect.height;

            // Height of each resizing element individually
            let panelCheechHeightPx = panelCheechBoundingClientRect.height;
            let panelCheechHeightPercent =
              (panelCheechHeightPx / totalHeightPx) * 100;

            let panelChongHeightPx = panelChongBoundingClientRect.height;
            let panelChongHeightPercent =
              (panelChongHeightPx / totalHeightPx) * 100;
            // Total height of the resizing elements
            let resizePanelsHeightPx = panelCheechHeightPx + panelChongHeightPx;
            let resizePanelsHeightPercent =
              (resizePanelsHeightPx / totalHeightPx) * 100;

            // Maximum and minimum size of a panel
            let scaleFactorDown = resizePanelsHeightPx / totalHeightPx;
            let scaleFactorUp = totalHeightPx / resizePanelsHeightPx;
            let maxHeightPx = 0.7 * totalHeightPx * scaleFactorDown;
            let maxHeightPercent = (maxHeightPx / totalHeightPx) * 100;
            let minHeightPx = 0.3 * totalHeightPx * scaleFactorUp;
            let minHeightPercent = (minHeightPx / totalHeightPx) * 100;

            // Mouse positions and direction
            const startingPositionMouseY = this.resizeEvent.startingPosition
              .clientY;
            const currentMouseY = event.clientY;
            let direction = "up";
            if (currentMouseY > startingPositionMouseY) {
              direction = "down";
            }

            // The delta traveled in pixels and as a percentage of the resizing
            // elements.
            const deltaPx =
              Math.abs(currentMouseY - startingPositionMouseY) *
              scaleFactorDown;
            const deltaPercent = (deltaPx / resizePanelsHeightPx) * 100;
            this.resizeEvent.startingPosition.clientY = currentMouseY;

            let newCheechHeightPercent: number;
            let newCheechHeightPx: number;
            let newChongHeightPercent: number;
            let newChongHeightPx: number;

            if (direction == "up") {
              newCheechHeightPercent = panelCheechHeightPercent - deltaPercent;
              newCheechHeightPx = panelCheechHeightPx - deltaPx;
              newChongHeightPercent = panelChongHeightPercent + deltaPercent;
              newChongHeightPx = panelChongHeightPx + deltaPx;
            } else {
              newCheechHeightPercent = panelCheechHeightPercent + deltaPercent;
              newCheechHeightPx = panelCheechHeightPx + deltaPx;
              newChongHeightPercent = panelChongHeightPercent - deltaPercent;
              newChongHeightPx = panelChongHeightPx - deltaPx;
            }
            if (newCheechHeightPercent > maxHeightPercent) {
              newCheechHeightPercent = maxHeightPercent;
              newChongHeightPercent = minHeightPercent;
            } else if (newCheechHeightPercent < minHeightPercent) {
              newCheechHeightPercent = minHeightPercent;
              newChongHeightPercent = maxHeightPercent;
            } else if (newChongHeightPercent > maxHeightPercent) {
              newChongHeightPercent = maxHeightPercent;
              newCheechHeightPercent = minHeightPercent;
            } else if (newChongHeightPercent < minHeightPercent) {
              newChongHeightPercent = minHeightPercent;
              newCheechHeightPercent = maxHeightPercent;
            }
            const panelContainer = this;
            requestAnimationFrame(function() {
              panelCheechElem.setAttribute(
                "style",
                `height: ${newCheechHeightPercent}%;`,
              );
              panelChongElem.setAttribute(
                "style",
                `height: ${newChongHeightPercent}%;`,
              );
              panelContainer.ticking = false;

              // Alex WIP
              PanelEventBus.$emit("panel-layout-update", {
                updated: true,
              });
            });
          } else {
            // Total Width of the container - for figuring out percentages.
            let totalWidthPx = containerBoundingClientRect.width;

            // Width of each resizing element individually
            let panelCheechWidthPx = panelCheechBoundingClientRect.width;
            let panelCheechWidthPercent =
              (panelCheechWidthPx / totalWidthPx) * 100;

            let panelChongWidthPx = panelChongBoundingClientRect.width;
            let panelChongWidthPercent =
              (panelChongWidthPx / totalWidthPx) * 100;
            // Total Width of the resizing elements
            let resizePanelsWidthPx = panelCheechWidthPx + panelChongWidthPx;
            let resizePanelsWidthPercent =
              (resizePanelsWidthPx / totalWidthPx) * 100;

            // Maximum and minimum size of a panel
            let scaleFactorDown = resizePanelsWidthPx / totalWidthPx;
            let scaleFactorUp = totalWidthPx / resizePanelsWidthPx;
            let maxWidthPx = 0.7 * totalWidthPx * scaleFactorDown;
            let maxWidthPercent = (maxWidthPx / totalWidthPx) * 100;
            let minWidthPx = 0.3 * totalWidthPx * scaleFactorUp;
            let minWidthPercent = (minWidthPx / totalWidthPx) * 100;

            // Mouse positions and direction
            const startingPositionMouseX = this.resizeEvent.startingPosition
              .clientX;
            const currentMouseX = event.clientX;
            let direction = "left";
            if (currentMouseX > startingPositionMouseX) {
              direction = "right";
            }

            // The delta traveled in pixels and as a percentage of the resizing
            // elements.
            const deltaPx =
              Math.abs(currentMouseX - startingPositionMouseX) *
              scaleFactorDown;
            const deltaPercent = (deltaPx / resizePanelsWidthPx) * 100;
            this.resizeEvent.startingPosition.clientX = currentMouseX;

            let newCheechWidthPercent: number;
            let newCheechWidthPx: number;
            let newChongWidthPercent: number;
            let newChongWidthPx: number;

            if (direction == "left") {
              newCheechWidthPercent = panelCheechWidthPercent - deltaPercent;
              newCheechWidthPx = panelCheechWidthPx - deltaPx;
              newChongWidthPercent = panelChongWidthPercent + deltaPercent;
              newChongWidthPx = panelChongWidthPx + deltaPx;
            } else {
              newCheechWidthPercent = panelCheechWidthPercent + deltaPercent;
              newCheechWidthPx = panelCheechWidthPx + deltaPx;
              newChongWidthPercent = panelChongWidthPercent - deltaPercent;
              newChongWidthPx = panelChongWidthPx - deltaPx;
            }
            if (newCheechWidthPercent > maxWidthPercent) {
              newCheechWidthPercent = maxWidthPercent;
              newChongWidthPercent = minWidthPercent;
            } else if (newCheechWidthPercent < minWidthPercent) {
              newCheechWidthPercent = minWidthPercent;
              newChongWidthPercent = maxWidthPercent;
            } else if (newChongWidthPercent > maxWidthPercent) {
              newChongWidthPercent = maxWidthPercent;
              newCheechWidthPercent = minWidthPercent;
            } else if (newChongWidthPercent < minWidthPercent) {
              newChongWidthPercent = minWidthPercent;
              newCheechWidthPercent = maxWidthPercent;
            }
            const panelContainer = this;
            requestAnimationFrame(function() {
              panelCheechElem.setAttribute(
                "style",
                `width: ${newCheechWidthPercent}%;`,
              );
              panelChongElem.setAttribute(
                "style",
                `width: ${newChongWidthPercent}%;`,
              );
              panelContainer.ticking = false;
            });

            // Alex WIP
            PanelEventBus.$emit("panel-layout-update", {
              updated: true,
            });
          }
        }
      }
    },
    stopResize(event: MouseEvent) {
      this.resizeEvent = null;
      event.preventDefault();
      document.body.style.cursor = "auto";
      document.onmouseup = null;
      document.onmousemove = null;
      document.onselectstart = null;
    },
    getPanelHolderElementByIndex(panelIndex: number): HTMLElement | null {
      let panel = document.getElementById(this.panelHolderRefName(panelIndex));
      return panel;
    },
    getPanelElementByIndex(panelIndex: number): HTMLElement | null {
      let panel;
      if (this.panelContainer.panels[panelIndex].type == "panel") {
        panel = document.getElementById(this.panelRefName(panelIndex));
      } else {
        panel = document.getElementById(
          this.otherPanelContainerRefName(panelIndex),
        );
      }
      return panel;
    },
    startResize(panelIndex: number, event: MouseEvent) {
      event.preventDefault();
      if (this.panelContainer.orientation == "column") {
        document.body.style.cursor = "row-resize";
      } else {
        document.body.style.cursor = "col-resize";
      }
      let panelCheechIndex = panelIndex;
      let panelCheech = this.getPanelElementByIndex(panelCheechIndex);
      let panelChongIndex = panelIndex + 1;
      let panelChong = this.getPanelElementByIndex(panelChongIndex);

      if (panelCheech && panelChong) {
        const panelCheechSize = {
          panelIndex: panelCheechIndex,
          size: {
            heightPx: panelCheech.offsetHeight,
            widthPx: panelCheech.offsetWidth,
          },
        };
        const panelChongSize = {
          panelIndex: panelChongIndex,
          size: {
            heightPx: panelChong.offsetHeight,
            widthPx: panelChong.offsetWidth,
          },
        };
        this.resizeEvent = {
          panelCheech: panelCheechSize,
          panelChong: panelChongSize,
          startingPosition: {
            clientX: event.clientX,
            clientY: event.clientY,
          },
        };
        document.onselectstart = () => {
          return false;
        };
        document.onmouseup = this.stopResize;
        document.onmousemove = this.resizePanels;
      } else {
        throw new Error(`Cannot find panel cheech and chong - bug! ;P`);
      }
    },
    otherPanelContainerRefName(index: number): string {
      return `panelContainer-${this.prefix()}-${index}`;
    },
    panelContainerRefName(): string {
      return `panelContainer-${this.prefix()}`;
    },
    vueHolderContainerRefName(): string {
      return `vueHolderContainer-${this.prefix()}`;
    },
    panelHolderRefName(panelIndex: number): string {
      return `panel-holder-${this.prefix()}-${panelIndex}`;
    },
    panelRefName(panelIndex: number): string {
      return `panel-${this.prefix()}-${panelIndex}`;
    },
    resizerHolderClasses(): Record<string, any> {
      let classes: Record<string, any> = {
        flex: true,
        "flex-grow": true,
        "justify-center": true,
      };
      if (this.panelContainer.orientation == "column") {
        classes["flex-col"] = true;
      } else {
        classes["flex-row"] = true;
      }
      return classes;
    },
    resizerStyles(): Record<string, any> {
      let styles: Record<string, any> = {};
      if (this.panelContainer.orientation == "column") {
        styles["height"] = `${RESIZER_SIZE}px`;
        styles["min-height"] = `${RESIZER_SIZE}px`;
        styles["cursor"] = "row-resize";
      } else {
        styles["width"] = `${RESIZER_SIZE}px`;
        styles["min-width"] = `${RESIZER_SIZE}px`;
        styles["cursor"] = "col-resize";
      }
      return styles;
    },
    resizerClasses(): Record<string, any> {
      let classes: Record<string, any> = {};
      if (this.panelContainer.orientation == "column") {
        classes["w-full"] = true;
      } else {
        classes["h-full"] = true;
      }
      classes["bg-gray-700"] = true;
      return classes;
    },
    panelContainerClasses(): Record<string, any> {
      let classes: Record<string, any> = {};
      classes["bg-gray-900"] = true;
      classes["w-full"] = true;
      classes["h-full"] = true;
      if (this.panelContainer.orientation == "column") {
        classes["flex"] = true;
        classes["flex-col"] = true;
      } else {
        classes["flex"] = true;
        classes["flex-row"] = true;
      }
      // Panel visibility
      classes["hidden"] = !this.isVisible;
      classes["overflow-hidden"] = !this.isVisible;
      return classes;
    },
    panelHolderClasses(panel: Panel): Record<string, any> {
      let classes: Record<string, any> = {};
      if (this.panelContainer.orientation == "column") {
        classes["flex"] = true;
        classes["flex-col"] = true;
      } else {
        classes["flex"] = true;
        classes["flex-row"] = true;
      }
      classes["content-between"] = true;
      return classes;
    },
    panelHolderStyles(): string {
      return "height: 100%; width: 100%;";
    },
    panelClasses(panel: Panel): Record<string, any> {
      let classes: Record<string, any> = {};
      classes["flex"] = true;
      //classes["bg-blue-500"] = true;
      return classes;
    },
    panelStyles(): string {
      let styles: string;
      styles = `height: ${100 - RESIZER_HEIGHT_PERCENTAGE}%; width: 100%;`;
      return styles;
    },
    minimizePanel(event: IMaximized) {
      if (this.panelContainerRefName() != event.panelContainerRef) {
        return;
      }

      let panelSelectors = this.$refs.panelSelector as Vue[];
      for (let i = 0; i < panelSelectors.length; i++) {
        // @ts-ignore
        if (panelSelectors[i].panelIndex != event.panelIndex) {
          const panelHolderId = this.panelHolderRefName(
            // @ts-ignore
            panelSelectors[i].panelIndex,
          );
          let panelHolderElem = document.getElementById(panelHolderId);
          if (panelHolderElem) {
            panelHolderElem.classList.remove("hidden");
            panelHolderElem.classList.remove("overflow-hidden");
          }
          // @ts-ignore
          panelSelectors[i].unhide();
        }
      }

      let ogPanelData = this.originalMaximizedElementData;
      if (this.maximizedData && ogPanelData.style) {
        let panelRef = this.maximizedData.panelRef;
        let panelElem = document.getElementById(panelRef) as HTMLElement;
        panelElem.classList.remove("absolute");

        let originalStyle = ogPanelData.style;
        if (originalStyle) {
          panelElem.setAttribute("style", originalStyle);
        }

        if (this.putBackTo.parent) {
          this.putBackTo.parent.prepend(panelElem);
        } else if (this.putBackTo.previousSibling) {
          this.putBackTo.previousSibling.insertAdjacentElement(
            "afterend",
            panelElem,
          );
        }
      }
    },
    maximizePanel(event: IMaximized) {
      if (this.panelContainerRefName() != event.panelContainerRef) {
        return;
      }
      let panelSelectors = this.$refs.panelSelector as Vue[];
      for (let i = 0; i < panelSelectors.length; i++) {
        // @ts-ignore
        if (panelSelectors[i].panelIndex != event.panelIndex) {
          const panelHolderId = this.panelHolderRefName(
            // @ts-ignore
            panelSelectors[i].panelIndex,
          );
          let panelHolderElem = document.getElementById(panelHolderId);

          if (panelHolderElem) {
            panelHolderElem.classList.add("hidden");
            panelHolderElem.classList.add("overflow-hidden");
          }
          // @ts-ignore
          panelSelectors[i].hide();
        }
      }

      let panelRef = event.panelRef;
      let panelContainerRef = event.panelContainerRef;
      let panelElem = document.getElementById(panelRef) as HTMLElement;
      let panelRootElem = document.getElementById(panelContainerRef);
      if (panelElem && panelRootElem) {
        let previousSibling = panelElem.previousElementSibling as HTMLElement;
        if (previousSibling) {
          this.putBackTo = { previousSibling };
        } else {
          let parent = panelElem.parentElement;
          if (parent) {
            this.putBackTo = { parent };
          } else {
            throw new Error("Cannot find element to put panel back to- bug!");
          }
        }
        this.originalMaximizedElementData = {
          style: _.clone(panelElem.getAttribute("style")),
        };
        this.maximizedData = event;
        panelRootElem.classList.add("relative");
        panelRootElem.prepend(panelElem);
        panelElem.classList.add("absolute");
        panelElem.setAttribute(
          "style",
          `position: absolute; width: 100%; height: 100%;`,
        );
      }
    },
    togglePanelVisibility() {
      this.isVisible = !this.isVisible;
    },
  },
});
</script>

<style scoped>
.resizer {
  /* background-color: #282F32; */
  background-color: #2b3336;
}
</style>
