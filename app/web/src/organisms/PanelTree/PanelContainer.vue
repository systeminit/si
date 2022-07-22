<template>
  <div :id="vueHolderContainerRefName()" class="flex flex-col w-full h-full">
    <!-- panel container -->
    <div :id="panelContainerRefName()" :class="panelContainerClasses()">
      <!-- panel holder -->
      <div
        v-for="(panel, panelIndex) in panelContainer.panels"
        :id="panelHolderRefName(panelIndex)"
        :key="panelIndex"
        :class="panelHolderClasses(panelIndex)"
        :style="panelHolderStyles(panelIndex)"
      >
        <!-- panel -->
        <div
          v-if="panel.type === 'panel'"
          :id="panelRefName(panelIndex)"
          :class="panelClasses(panelIndex)"
          :style="panelStyles()"
        >
          <PanelSelector
            ref="panelSelector"
            :panel-index="panelIndex"
            :panel-ref="panelRefName(panelIndex)"
            :panel-container-ref="panelContainerRefName()"
            :initial-panel-type="panel.name"
            :initial-panel-sub-type="panel.subType"
            @panel-maximize-container="maximizePanel($event)"
            @panel-minimize-container="minimizePanel($event)"
            @panel-maximize-full="maximizeFull($event)"
            @panel-minimize-full="minimizeFull($event)"
          />
        </div>
        <PanelContainer
          v-else
          :maximized-full-panel="maximizedFullPanel"
          :panel-container="panel"
          :index="panelIndex"
          :parent-prefix="prefix()"
          @panel-maximize-full="maximizeFull($event)"
          @panel-minimize-full="minimizeFull($event)"
        />
        <!-- resizer -->
        <div
          v-if="panelIndex !== panelContainer.panels.length - 1"
          :class="resizerHolderClasses()"
          style="min-width: 4px; min-height: 4px; z-index: 100"
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

<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import PanelSelector from "./PanelSelector.vue";
import {
  IPanelContainer,
  PanelMaximized,
  ResizeEvent,
  RESIZER_SIZE,
} from "./panel_types";
import {
  createPanelContainerMaximizedObservable,
  createPanelContainerSizeObservable,
  restorePanelContainerMaximizedObservable,
  restorePanelContainerSizeObservable,
} from "@/observable/editor";

const props = defineProps<{
  panelContainer: IPanelContainer;
  parentPrefix: string;
  index: number;
  maximizedFullPanel: PanelMaximized | null;
}>();

const emit = defineEmits(["panel-maximize-full", "panel-minimize-full"]);

const resizeEvent = ref<null | ResizeEvent>(null);
const ticking = ref<boolean>(false);
const maximizedData = ref<PanelMaximized | null>(null);
const panelSelector = ref<Array<typeof PanelSelector>>([]);
const panelSize = ref<
  Record<string, { width: number; height: number; hidden: boolean }>
>({});

onMounted(() => {
  setStaticSizes();
  setPanelIsMaximizedContainerEnabled();
  setInitialPanelSize();
  const panelContainerSize = restorePanelContainerSizeObservable(
    panelContainerRefName(),
  );
  if (panelContainerSize) {
    panelSize.value = panelContainerSize;
  }
  const panelContainerMaximized = restorePanelContainerMaximizedObservable(
    panelContainerRefName(),
  );
  if (panelContainerMaximized) {
    maximizedData.value = panelContainerMaximized;
  }
});

watch(
  () => props.maximizedFullPanel,
  (maximizedFullPanel) => {
    if (maximizedFullPanel) {
      if (
        !maximizedFullPanel.panelContainerRef.startsWith(
          panelContainerRefName(),
        )
      ) {
        for (const value of Object.values(panelSize.value)) {
          value.hidden = true;
        }
      }
    } else {
      for (const value of Object.values(panelSize.value)) {
        value.hidden = false;
      }
    }
  },
);

const prefix = () => {
  return `${props.parentPrefix}-${props.index}`;
};

const setStaticSizes = () => {
  const panelContainer = props.panelContainer;
  const numberOfPanels = panelContainer.panels.length;
  const startingPercentage = Math.floor(100 / numberOfPanels);
  for (
    let panelIndex = 0;
    panelIndex < panelContainer.panels.length;
    panelIndex++
  ) {
    const panelHolderElem = document.getElementById(
      panelHolderRefName(panelIndex),
    );
    if (panelHolderElem) {
      if (panelHolderElem && props.panelContainer.orientation == "column") {
        panelSize.value[`${panelIndex}`] = {
          height: startingPercentage,
          width: 100,
          hidden: false,
        };
      } else {
        panelSize.value[`${panelIndex}`] = {
          width: startingPercentage,
          height: 100,
          hidden: false,
        };
      }
    }
  }
};

const setPanelIsMaximizedContainerEnabled = () => {
  let panelSelectors = panelSelector.value;
  if (panelSelectors && panelSelectors.length > 1) {
    for (let i = 0; i < panelSelectors.length; i++) {
      panelSelectors[i].isMaximizedContainerEnabled = true;
    }
  } else {
    if (panelSelectors && panelSelectors[0]) {
      panelSelectors[0].isMaximizedContainerEnabled = false;
    }
  }
};

const setInitialPanelSize = () => {
  // Temporary hack to see how the layout feels.
  const panelContainer = props.panelContainer;
  const orientation = panelContainer.orientation;

  const panelCheechIndex = 0;
  const panelChongIndex = 1;

  const panelContainerElem = document.getElementById(panelContainerRefName());
  const panelCheechElem = getPanelHolderElementByIndex(panelCheechIndex);
  const panelChongElem = getPanelHolderElementByIndex(panelChongIndex);

  if (orientation == "row") {
    if (panelContainerElem && panelCheechElem && panelChongElem) {
      // @ts-ignore
      if (panelContainer.panels[0].width) {
        // @ts-ignore
        const newCheechWidthPercent = panelContainer.panels[0].width;
        const newChongWidthPercent = 100 - newCheechWidthPercent;
        if (panelSize.value[`${panelCheechIndex}`]) {
          panelSize.value[`${panelCheechIndex}`]["width"] =
            newCheechWidthPercent;
        }
        if (panelSize.value[`${panelChongIndex}`]) {
          panelSize.value[`${panelChongIndex}`]["width"] = newChongWidthPercent;
        }
      }
    }
  }
};

const resizePanels = (event: MouseEvent) => {
  event.preventDefault();
  if (resizeEvent.value && !ticking.value) {
    ticking.value = true;

    const panelContainer = props.panelContainer;
    const orientation = panelContainer.orientation;

    const panelContainerElem = document.getElementById(panelContainerRefName());
    const panelCheechIndex = resizeEvent.value.panelCheech.panelIndex;
    const panelChongIndex = resizeEvent.value.panelChong.panelIndex;
    const panelCheechElem = getPanelHolderElementByIndex(
      resizeEvent.value.panelCheech.panelIndex,
    );
    const panelChongElem = getPanelHolderElementByIndex(
      resizeEvent.value.panelChong.panelIndex,
    );

    if (panelContainerElem && panelCheechElem && panelChongElem) {
      let containerBoundingClientRect =
        panelContainerElem.getBoundingClientRect();
      let panelCheechBoundingClientRect =
        panelCheechElem.getBoundingClientRect();
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

        // Maximum and minimum size of a panel
        let scaleFactorDown = resizePanelsHeightPx / totalHeightPx;
        let scaleFactorUp = totalHeightPx / resizePanelsHeightPx;
        let maxHeightPx = 0.7 * totalHeightPx * scaleFactorDown;
        let maxHeightPercent = (maxHeightPx / totalHeightPx) * 100;
        let minHeightPx = 0.3 * totalHeightPx * scaleFactorUp;
        let minHeightPercent = (minHeightPx / totalHeightPx) * 100;

        // Mouse positions and direction
        const startingPositionMouseY =
          resizeEvent.value.startingPosition.clientY;
        const currentMouseY = event.clientY;
        let direction = "up";
        if (currentMouseY > startingPositionMouseY) {
          direction = "down";
        }

        // The delta traveled in pixels and as a percentage of the resizing
        // elements.
        const deltaPx =
          Math.abs(currentMouseY - startingPositionMouseY) * scaleFactorDown;
        const deltaPercent = (deltaPx / resizePanelsHeightPx) * 100;
        resizeEvent.value.startingPosition.clientY = currentMouseY;

        let newCheechHeightPercent: number;
        let newChongHeightPercent: number;

        if (direction == "up") {
          newCheechHeightPercent = panelCheechHeightPercent - deltaPercent;
          newChongHeightPercent = panelChongHeightPercent + deltaPercent;
        } else {
          newCheechHeightPercent = panelCheechHeightPercent + deltaPercent;
          newChongHeightPercent = panelChongHeightPercent - deltaPercent;
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
        requestAnimationFrame(function () {
          if (panelSize.value[`${panelCheechIndex}`]) {
            panelSize.value[`${panelCheechIndex}`]["height"] =
              newCheechHeightPercent;
          }
          if (panelSize.value[`${panelChongIndex}`]) {
            panelSize.value[`${panelChongIndex}`]["height"] =
              newChongHeightPercent;
          }
          ticking.value = false;
        });
      } else {
        // Total Width of the container - for figuring out percentages.
        let totalWidthPx = containerBoundingClientRect.width;

        // Width of each resizing element individually
        let panelCheechWidthPx = panelCheechBoundingClientRect.width;
        let panelCheechWidthPercent = (panelCheechWidthPx / totalWidthPx) * 100;

        let panelChongWidthPx = panelChongBoundingClientRect.width;
        let panelChongWidthPercent = (panelChongWidthPx / totalWidthPx) * 100;
        // Total Width of the resizing elements
        let resizePanelsWidthPx = panelCheechWidthPx + panelChongWidthPx;

        // Maximum and minimum size of a panel
        let scaleFactorDown = resizePanelsWidthPx / totalWidthPx;
        let scaleFactorUp = totalWidthPx / resizePanelsWidthPx;
        let maxWidthPx = 0.7 * totalWidthPx * scaleFactorDown;
        let maxWidthPercent = (maxWidthPx / totalWidthPx) * 100;
        let minWidthPx = 0.3 * totalWidthPx * scaleFactorUp;
        let minWidthPercent = (minWidthPx / totalWidthPx) * 100;

        // Mouse positions and direction
        const startingPositionMouseX =
          resizeEvent.value.startingPosition.clientX;
        const currentMouseX = event.clientX;
        let direction = "left";
        if (currentMouseX > startingPositionMouseX) {
          direction = "right";
        }

        // The delta traveled in pixels and as a percentage of the resizing
        // elements.
        const deltaPx =
          Math.abs(currentMouseX - startingPositionMouseX) * scaleFactorDown;
        const deltaPercent = (deltaPx / resizePanelsWidthPx) * 100;
        resizeEvent.value.startingPosition.clientX = currentMouseX;

        let newCheechWidthPercent: number;
        let newChongWidthPercent: number;

        if (direction == "left") {
          newCheechWidthPercent = panelCheechWidthPercent - deltaPercent;
          newChongWidthPercent = panelChongWidthPercent + deltaPercent;
        } else {
          newCheechWidthPercent = panelCheechWidthPercent + deltaPercent;
          newChongWidthPercent = panelChongWidthPercent - deltaPercent;
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
        requestAnimationFrame(function () {
          if (panelSize.value[`${panelCheechIndex}`]) {
            panelSize.value[`${panelCheechIndex}`]["width"] =
              newCheechWidthPercent;
          }

          if (panelSize.value[`${panelChongIndex}`]) {
            panelSize.value[`${panelChongIndex}`]["width"] =
              newChongWidthPercent;
          }

          ticking.value = false;
        });
      }
    }
  }
};

const stopResize = (event: MouseEvent) => {
  resizeEvent.value = null;
  event.preventDefault();
  document.body.style.cursor = "auto";
  document.onmouseup = null;
  document.onmousemove = null;
  document.onselectstart = null;
};

const getPanelHolderElementByIndex = (panelIndex: number) => {
  return document.getElementById(panelHolderRefName(panelIndex));
};

const getPanelElementByIndex = (panelIndex: number) => {
  let panel;
  if (props.panelContainer.panels[panelIndex].type == "panel") {
    panel = document.getElementById(panelRefName(panelIndex));
  } else {
    panel = document.getElementById(otherPanelContainerRefName(panelIndex));
  }
  return panel;
};

const startResize = (panelIndex: number, event: MouseEvent) => {
  event.preventDefault();
  if (props.panelContainer.orientation == "column") {
    document.body.style.cursor = "row-resize";
  } else {
    document.body.style.cursor = "col-resize";
  }
  let panelCheechIndex = panelIndex;
  let panelCheech = getPanelElementByIndex(panelCheechIndex);
  let panelChongIndex = panelIndex + 1;
  let panelChong = getPanelElementByIndex(panelChongIndex);

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
    resizeEvent.value = {
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
    document.onmouseup = stopResize;
    document.onmousemove = resizePanels;
  } else {
    throw new Error(`Cannot find panel cheech and chong - bug! ;P`);
  }
};

const otherPanelContainerRefName = (index: number) => {
  return `panelContainer-${prefix()}-${index}`;
};

const panelContainerRefName = () => {
  return `panelContainer-${prefix()}`;
};

const vueHolderContainerRefName = () => {
  return `vueHolderContainer-${prefix()}`;
};

const panelHolderRefName = (panelIndex: number) => {
  return `panel-holder-${prefix()}-${panelIndex}`;
};

const panelRefName = (panelIndex: number) => {
  return `panel-${prefix()}-${panelIndex}`;
};

const resizerHolderClasses = () => {
  let classes: Record<string, boolean> = {
    flex: true,
    "flex-grow": true,
    "justify-center": true,
  };
  if (maximizedData.value) {
    classes["hidden"] = true;
    classes["overflow-hidden"] = true;
  }
  if (props.panelContainer.orientation == "column") {
    classes["flex-col"] = true;
  } else {
    classes["flex-row"] = true;
  }
  return classes;
};

const resizerStyles = () => {
  let styles: Record<string, string> = {};
  if (props.panelContainer.orientation == "column") {
    styles["height"] = `${RESIZER_SIZE}px`;
    styles["min-height"] = `${RESIZER_SIZE}px`;
    styles["cursor"] = "row-resize";
  } else {
    styles["width"] = `${RESIZER_SIZE}px`;
    styles["min-width"] = `${RESIZER_SIZE}px`;
    styles["cursor"] = "col-resize";
  }
  return styles;
};

const resizerClasses = () => {
  let classes: Record<string, true> = {};
  if (props.panelContainer.orientation == "column") {
    classes["w-full"] = true;
  } else {
    classes["h-full"] = true;
  }
  classes["resizer"] = true;
  return classes;
};

const panelContainerClasses = () => {
  let classes: Record<string, boolean> = {};
  classes["bg-gray-900"] = true;
  classes["w-full"] = true;
  classes["h-full"] = true;
  if (props.panelContainer.orientation == "column") {
    classes["flex"] = true;
    classes["flex-col"] = true;
  } else {
    classes["flex"] = true;
    classes["flex-row"] = true;
  }
  return classes;
};

const panelHolderClasses = (panelIndex: number) => {
  let classes: Record<string, boolean> = {};
  if (
    panelSize.value[`${panelIndex}`] &&
    panelSize.value[`${panelIndex}`].hidden
  ) {
    classes["hidden"] = true;
    classes["overflow-hidden"] = true;
  }
  if (props.panelContainer.orientation == "column") {
    classes["flex"] = true;
    classes["flex-col"] = true;
  } else {
    classes["flex"] = true;
    classes["flex-row"] = true;
  }
  classes["content-between"] = true;
  return classes;
};

const panelHolderStyles = (panelIndex: number) => {
  const size = panelSize.value[`${panelIndex}`];
  if (size) {
    if (size.hidden) {
      return `display: none;`;
    } else {
      if (
        maximizedData.value &&
        maximizedData.value.panelContainerRef.startsWith(
          panelContainerRefName(),
        )
      ) {
        return `height: 100%; width: 100%;`;
      } else {
        return `height: ${size["height"]}%; width: ${size["width"]}%;`;
      }
    }
  } else {
    return "height: 100%; width: 100%;";
  }
};

const panelClasses = (panelIndex: number) => {
  let classes: Record<string, boolean> = {};
  if (
    panelSize.value[`${panelIndex}`] &&
    panelSize.value[`${panelIndex}`].hidden
  ) {
    classes["hidden"] = true;
    classes["overflow-hidden"] = true;
  }
  classes["flex"] = true;
  return classes;
};

const panelStyles = () => {
  let styles: string;
  styles = `height: 100%; width: 100%;`;
  return styles;
};

const minimizePanel = (_event: PanelMaximized) => {
  maximizedData.value = null;
  for (const value of Object.values(panelSize.value)) {
    value.hidden = false;
  }
};

const maximizePanel = (event: PanelMaximized) => {
  maximizedData.value = event;
  for (const [key, value] of Object.entries(panelSize.value)) {
    value.hidden = key != `${event.panelIndex}`;
  }
};

const maximizeFull = (event: PanelMaximized) => {
  if (event.panelContainerRef === panelContainerRefName()) {
    maximizePanel(event);
  } else {
    // We are dealing with the case where we are a parent panel
    // container. In that case, if we hold the panel we are maximizing,
    // we want to make sure we are not hidden.
    //
    // This code only works because the grid is 4x4.
    if (event.panelContainerRef.startsWith(panelContainerRefName())) {
      maximizedData.value = event;
      const showPanelIndex = event.panelContainerRef.slice(-1);
      for (const [key, value] of Object.entries(panelSize.value)) {
        value.hidden = key !== showPanelIndex;
      }
    } else {
      maximizedData.value = event;
      for (const value of Object.values(panelSize.value)) {
        value.hidden = true;
      }
    }
  }
  emit("panel-maximize-full", event);
};

const minimizeFull = (event: PanelMaximized) => {
  minimizePanel(event);
  emit("panel-minimize-full", event);
};

const panelSizeObservable$ = createPanelContainerSizeObservable(
  panelContainerRefName(),
);

const panelMaximizedObservable$ = createPanelContainerMaximizedObservable(
  panelContainerRefName(),
);

watch(
  panelSize,
  (panelSize) => {
    panelSizeObservable$.next(panelSize);
  },
  { deep: true },
);

watch(
  maximizedData,
  (panelMaximized) => {
    panelMaximizedObservable$.next(panelMaximized);
  },
  { deep: true },
);
</script>

<style scoped>
.resizer {
  background-color: #404040;
}
</style>
