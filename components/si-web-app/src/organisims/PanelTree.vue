<template>
  <div
    class="flex flex-col w-full h-full"
    id="panelTreeRoot"
    @mouseenter="activateShortcuts()"
    @mouseleave="deactivateShortcuts()"
  >
    <PanelContainer
      v-for="(panelContainer, panelContainerIndex) in panelContainers"
      :key="panelContainerIndex"
      :panelContainer="panelContainer"
      parentPrefix="root"
      :index="panelContainerIndex"
      @maximize-full="maximizePanelFull($event)"
    />
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import PanelContainer, {
  IPanelContainer,
} from "@/molecules/PanelContainer.vue";
import { PanelEventBus } from "@/atoms/PanelEventBus";
import _ from "lodash";

interface IData {
  panelContainers: IPanelContainer[];
  originalMaximizedElementData: {
    style: string | null;
  };
  putBackTo: {
    parent?: string;
    previousSibling?: string;
  };
  maximizedData: IMaximized | null;
  shortcuts: boolean;
}

export interface IMaximized {
  panelRef: string;
  panelContainerRef: string;
}

export default Vue.extend({
  name: "PanelTree",
  components: {
    PanelContainer,
  },
  created() {
    PanelEventBus.$on("shortcut", this.handleShortcut);
    PanelEventBus.$on("maximize-full", this.maximizePanelFull);
    PanelEventBus.$on("minimize-full", this.minimizePanelFull);
    PanelEventBus.$on("create-new-panel", this.createNewPanel);
    PanelEventBus.$on("delete-panel", this.deletePanel);
  },
  beforeDestroy() {
    PanelEventBus.$off("shortcut", this.handleShortcut);
    PanelEventBus.$off("maximize-full", this.maximizePanelFull);
    PanelEventBus.$off("minimize-full", this.minimizePanelFull);
    PanelEventBus.$off("create-new-panel", this.createNewPanel);
    PanelEventBus.$off("delete-panel", this.deletePanel);
  },
  data(): IData {
    return {
      shortcuts: false,
      originalMaximizedElementData: {
        style: null,
      },
      maximizedData: null,
      putBackTo: {},
      panelContainers: [
        {
          orientation: "row",
          type: "panelContainer",
          panels: [
            {
              orientation: "column",
              type: "panelContainer",
              width: 67,
              panels: [
                {
                  name: "schematic",
                  type: "panel",
                },
                {
                  name: "schematic",
                  type: "panel",
                },
              ],
            },
            {
              orientation: "column",
              type: "panelContainer",
              panels: [
                {
                  name: "attribute",
                  type: "panel",
                },
                {
                  name: "schematic",
                  type: "panel",
                },
              ],
            },
            //{
            //  orientation: "row",
            //  type: "panelContainer",
            //  panels: [
            //    {
            //      name: "schematic",
            //      type: "panel",
            //    },
            //    {
            //      orientation: "column",
            //      type: "panelContainer",
            //      panels: [
            //        {
            //          name: "attribute",
            //          type: "panel",
            //        },
            //        {
            //          name: "somethingelse",
            //          type: "panel",
            //        },
            //      ],
            //    },
            //  ],
            //},
          ],
        },
      ],
    };
  },
  methods: {
    createNewPanel(event: { panelContainerRef: string }) {
      let panelContainerRefParts = event.panelContainerRef.split("-");
      let pc = this.panelContainers[0];
      for (let x = 3; x < panelContainerRefParts.length; x++) {
        let index: number = parseInt(panelContainerRefParts[x], 10);
        if (pc.panels[index] && pc.panels[index].type == "panelContainer") {
          // @ts-ignore
          pc = pc.panels[index];
        }
      }
      if (pc) {
        if (pc.panels.length < 2) {
          pc.panels.push({ name: "schematic", type: "panel" });
        }
      }
      PanelEventBus.$emit("panel-created", event);
    },
    // Bug - panel delete does the wrong panel, always the last one.
    deletePanel(event: { panelRef: string }) {
      let panelRefParts = event.panelRef.split("-");
      let pc = this.panelContainers[0];
      let index: number;
      for (let x = 3; x < panelRefParts.length; x++) {
        index = parseInt(panelRefParts[x], 10);
        if (pc.panels[index] && pc.panels[index].type == "panelContainer") {
          // @ts-ignore
          pc = pc.panels[index];
        }
        if (x == panelRefParts.length - 1) {
          if (pc.panels.length > 1) {
            pc.panels.splice(index, 1);
          }
        }
      }
      PanelEventBus.$emit("panel-deleted", event);
    },
    handleShortcut(event: KeyboardEvent) {
      if (this.shortcuts) {
        if (event.altKey && event.shiftKey && event.key == "C") {
          if (this.panelContainers[0].panels.length < 2) {
            this.panelContainers[0].panels.push({
              orientation: "column",
              type: "panelContainer",
              panels: [{ name: "new", type: "panel" }],
            });
          }
        } else if (event.altKey && event.shiftKey && event.key == "R") {
          if (this.panelContainers[0].panels.length < 2) {
            this.panelContainers[0].panels.push({
              orientation: "row",
              type: "panelContainer",
              panels: [{ name: "new", type: "panel" }],
            });
          }
        }
      }
    },
    activateShortcuts() {
      this.shortcuts = true;
    },
    deactivateShortcuts() {
      this.shortcuts = false;
    },
    minimizePanelFull(event: IMaximized) {
      let ogPanelData = this.originalMaximizedElementData;
      if (this.maximizedData && ogPanelData.style) {
        let panelRef = this.maximizedData.panelRef;
        let panelElem = document.getElementById(panelRef) as HTMLElement;
        panelElem.classList.remove("absolute");

        // panelElem.classList.remove("z-100");
        if (
          panelElem.nextElementSibling &&
          panelElem.nextElementSibling!.classList
        ) {
          panelElem.nextElementSibling!.classList.remove("hidden");
          panelElem.nextElementSibling!.classList.remove("overflow-hidden");
        }

        let originalStyle = ogPanelData.style;
        if (originalStyle) {
          panelElem.setAttribute("style", originalStyle);
        }

        if (this.putBackTo.parent) {
          let parent = document.getElementById(
            this.putBackTo.parent,
          ) as HTMLElement;
          parent.prepend(panelElem);
        } else if (this.putBackTo.previousSibling) {
          let previousSibling = document.getElementById(
            this.putBackTo.previousSibling,
          ) as HTMLElement;
          previousSibling.insertAdjacentElement("afterend", panelElem);
        }
      }
    },
    maximizePanelFull(event: IMaximized) {
      let panelRef = event.panelRef;
      let panelContainerRef = event.panelContainerRef;
      let panelElem = document.getElementById(panelRef) as HTMLElement;
      let panelTreeRootElem = document.getElementById("panelTreeRoot");
      if (panelElem && panelTreeRootElem) {
        let previousSibling = panelElem.previousElementSibling as HTMLElement;
        if (previousSibling) {
          this.putBackTo = { previousSibling: previousSibling.id };
        } else {
          let parent = panelElem.parentElement;
          if (parent) {
            this.putBackTo = { parent: parent.id };
          } else {
            throw new Error("Cannot find element to put panel back to- bug!");
          }
        }
        this.originalMaximizedElementData = {
          style: _.clone(panelElem.getAttribute("style")),
        };
        this.maximizedData = event;
        let panelTreeRootWidth = Math.floor(
          panelTreeRootElem.getBoundingClientRect().width,
        );
        let panelTreeRootHeight = Math.floor(
          panelTreeRootElem.getBoundingClientRect().height,
        );
        panelTreeRootElem.classList.add("relative");
        panelTreeRootElem.prepend(panelElem);
        panelElem.classList.add("absolute");

        // Hide the other panels
        // panelElem.classList.add("z-100");

        // Should loop over all sibling. We only have one for now so ....
        // We should do this directly at the component level instead of here
        panelElem.nextElementSibling!.classList.add("hidden");
        panelElem.nextElementSibling!.classList.add("overflow-hidden");

        panelElem.setAttribute(
          "style",
          `position: absolute; width: 100%; height: 100%;`,
        );
      }
    },
  },
});
</script>
