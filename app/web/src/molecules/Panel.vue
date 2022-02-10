<template>
  <div
    v-if="isVisible"
    class="flex flex-col w-full h-full"
    :class="panelClasses"
    @mouseenter="mouseEnter()"
    @mouseleave="mouseLeave()"
  >
    <div
      class="flex flex-row items-center w-full bg-black"
      :class="panelMenuClasses"
      style="height: 2.5rem; min-height: 2.5rem"
    >
      <div class="flex justify-start">
        <div class="min-w-max">
          <SiSelect
            id="selectPanelType"
            v-model="selectedPanelType"
            size="xs"
            :options="panelTypes"
            class="pl-2"
            :styling="panelSelectorStyling()"
            @change="changePanelType"
          />
        </div>
      </div>
      <div class="flex justify-start flex-grow">
        <slot name="menuButtons"></slot>
      </div>
      <div class="flex flex-row items-center justify-end flex-grow">
        <div class="flex items-center h-full pr-2">
          <button
            v-if="maximizedContainer && !maximizedFull"
            class="flex items-center"
            data-testid="minimize-container"
            @click="minimizeContainer"
          >
            <VueFeather type="minimize-2" size="1.2rem" stroke-width="1.5" />
          </button>

          <button
            v-if="
              !maximizedContainer &&
              !maximizedFull &&
              isMaximizedContainerEnabled
            "
            class="flex items-center"
            data-testid="maximize-container"
            @click="maximizeContainer"
          >
            <VueFeather type="maximize-2" size="1.2rem" stroke-width="1.5" />
          </button>
        </div>
        <div class="flex items-center h-full pr-2">
          <button
            v-if="maximizedFull"
            class="flex items-center"
            data-testid="minimize-full"
            @click="minimizeFull"
          >
            <VueFeather type="minimize" size="1.2rem" stroke-width="1.5" />
          </button>

          <button
            v-else
            class="flex items-center"
            data-testid="maximize-full"
            @click="maximizeFull"
          >
            <VueFeather type="maximize" size="1.2rem" stroke-width="1.5" />
          </button>
        </div>
      </div>
    </div>
    <div class="flex flex-grow w-full h-full overflow-auto">
      <slot name="content"></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import SiSelect from "@/atoms/SiSelect.vue";
import { PanelMaximized, PanelType } from "@/organisims/PanelTree/panel_types";
import { LabelList } from "@/api/sdf/dal/label_list";
import VueFeather from "vue-feather";
import { computed, onMounted, PropType, ref, watch } from "vue";
import {
  createPanelMaximizedContainerObservable,
  createPanelMaximizedFullObservable,
  restorePanelMaximizedContainerObservable,
  restorePanelMaximizedFullObservable,
} from "@/observable/editor";
import _ from "lodash";

const props = defineProps({
  panelIndex: { type: Number, required: true },
  panelRef: { type: String, required: true },
  panelContainerRef: { type: String, required: true },
  initialPanelType: {
    type: String as PropType<PanelType>,
    default: PanelType.Empty,
  },
  initialMaximizedFull: Boolean,
  initialMaximizedContainer: Boolean,
  isVisible: { type: Boolean, default: true },
  isMaximizedContainerEnabled: Boolean,
});

const selectedPanelType = ref<PanelType>(props.initialPanelType);
const maximizedFull = ref(props.initialMaximizedFull);
const maximizedContainer = ref(props.initialMaximizedContainer);
const isActive = ref<boolean>(false);

const maximizedFull$ = createPanelMaximizedFullObservable(props.panelRef);
watch(maximizedFull, (mf) => {
  maximizedFull$.next(mf);
});
const maximizedContainer$ = createPanelMaximizedContainerObservable(
  props.panelRef,
);
watch(maximizedContainer, (mc) => {
  maximizedContainer$.next(mc);
});

onMounted(() => {
  const maximizedFullData = restorePanelMaximizedFullObservable(props.panelRef);
  if (!_.isNull(maximizedFullData)) {
    maximizedFull.value = maximizedFullData;
  }
  const maximizedContainerData = restorePanelMaximizedContainerObservable(
    props.panelRef,
  );
  if (!_.isNull(maximizedContainerData)) {
    maximizedContainer.value = maximizedContainerData;
  }
});

const panelTypes = computed<LabelList<PanelType>>(() => {
  return [
    {
      label: "Schematic",
      value: PanelType.Schematic,
    },
    {
      label: "Attribute",
      value: PanelType.Attribute,
    },
    {
      label: "Secret",
      value: PanelType.Secret,
    },
    {
      label: "Empty",
      value: PanelType.Empty,
    },
  ];
});

const emits = defineEmits([
  "change-panel",
  "panel-maximize-container",
  "panel-minimize-container",
  "panel-maximize-full",
  "panel-minimize-full",
]);

const panelSelectorStyling = () => {
  let classes: Record<string, boolean> = {};
  classes["bg-selectordark"] = true;
  classes["text-gray-400"] = true;
  classes["border-gray-800"] = true;
  return classes;
};

const changePanelType = () => {
  emits("change-panel", selectedPanelType.value);
};

const mouseEnter = () => {
  isActive.value = true;
};

const mouseLeave = () => {
  isActive.value = false;
};

function sizeEventData(): PanelMaximized {
  return {
    panelIndex: props.panelIndex,
    panelRef: props.panelRef,
    panelContainerRef: props.panelContainerRef,
  };
}

const maximizeContainer = () => {
  maximizedContainer.value = true;
  emits("panel-maximize-container", sizeEventData());
};
const minimizeContainer = () => {
  maximizedContainer.value = false;
  emits("panel-minimize-container", sizeEventData());
};
const maximizeFull = () => {
  maximizedFull.value = true;
  maximizedContainer.value = true;
  emits("panel-maximize-full", sizeEventData());
};
const minimizeFull = () => {
  maximizedFull.value = false;
  maximizedContainer.value = false;
  emits("panel-minimize-full", sizeEventData());
};

const panelClasses = computed(() => {
  let classes: Record<string, boolean> = {};
  classes["inactive-panel"] = !isActive.value;
  return classes;
});
const panelMenuClasses = computed(() => {
  let classes: Record<string, boolean> = {};
  classes["inactive-panel-menu"] = !isActive.value;
  return classes;
});
</script>

<style scoped>
div.inactive-panel-menu > * {
  filter: brightness(90%);
}

/*
div.active-panel {
  border: solid;
  border-color: #323536;
  border-width: 0.1em;
}

div.inactive-panel > * {
  filter: brightness(98%);
}
*/
</style>
