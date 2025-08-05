<template>
  <div
    ref="inputWindowRef"
    :class="
      clsx(
        'mx-xs pb-sm px-sm flex-col flex items-center gap-xs [&>*]:w-full',
        themeClasses('bg-neutral-300', 'bg-neutral-700'),
        !inputOpen && 'h-[48px]',
      )
    "
  >
    <div class="flex flex-row items-center gap-sm">
      <TruncateWithTooltip class="py-sm grow max-w-fit text-sm">
        Add a component to be managed by "{{ parentComponentName }}"
      </TruncateWithTooltip>
      <input
        v-if="inputOpen"
        ref="inputRef"
        placeholder="Find and select a component"
        :class="
          clsx(
            'min-w-[300px] grow block h-lg text-sm font-mono p-xs',
            'border focus:outline-none focus:ring-0 focus:z-10',
            themeClasses(
              'text-black bg-white disabled:bg-neutral-100 border-action-500',
              'text-white bg-black disabled:bg-neutral-900 border-action-300',
            ),
          )
        "
        @input="(e) => onInputChange(e)"
        @blur="blur"
        @keydown.esc.stop.prevent="closeInput"
        @keydown.up.prevent="onUp"
        @keydown.down.prevent="onDown"
        @keydown.enter.prevent="createManagementConnection()"
      />
      <div
        v-else
        :class="
          clsx(
            'min-w-[300px] grow flex flex-row items-center gap-xs',
            'h-lg p-xs text-sm border font-mono cursor-text',
            themeClasses(
              'text-shade-100 bg-shade-0 border-neutral-400',
              'text-shade-0 bg-shade-100 border-neutral-600',
            ),
          )
        "
        @click="openInput"
      >
        <Icon name="search" size="sm" class="flex-none" />
        <div class="grow">Find and select components</div>
        <Icon name="chevron--down" size="sm" class="flex-none" />
      </div>
    </div>
    <div v-if="inputOpen" class="flex flex-col items-stretch">
      <EmptyState
        v-if="filteredComponents.length === 0"
        :icon="activeFilterStr.length === 0 ? 'search' : 'x'"
        :text="
          activeFilterStr.length === 0
            ? 'Type to search for a component to manage'
            : 'No components match your search'
        "
      />
      <ManagementConnectionCard
        v-for="(component, index) in filteredComponents"
        v-else
        :key="component.id"
        :componentId="component.id"
        selectable
        :selected="index === selectedOptionIndex"
        @mouseover="selectedOptionIndex = index"
        @select="createManagementConnection(component)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import clsx from "clsx";
import {
  Icon,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import { debounce } from "lodash-es";
import { computed, nextTick, PropType, reactive, ref, watch } from "vue";
import { Fzf } from "fzf";
import { MouseDetails, mouseEmitter } from "./logic_composables/emitters";
import EmptyState from "./EmptyState.vue";
import ManagementConnectionCard from "./ManagementConnectionCard.vue";
import { SimpleConnection } from "./layout_components/ConnectionLayout.vue";
import { routes, useApi } from "./api_composables";
import { UpdateComponentManageArgs } from "./api_composables/component";
import { useContext } from "./logic_composables/context";

export type PossibleConnectionComponent = {
  id: string;
  name?: string;
  schemaVariantName?: string;
};

const ctx = useContext();

const props = defineProps({
  existingEdges: {
    type: Array as PropType<SimpleConnection[]>,
    required: true,
  },
  parentComponentName: { type: String, required: true },
  parentComponentId: { type: String, required: true },
});

const inputRef = ref<HTMLInputElement>();
const inputWindowRef = ref<HTMLDivElement>();

const onMouseDown = (e: MouseDetails["mousedown"]) => {
  const target = e.target;
  if (!(target instanceof Element)) {
    return;
  }
  if (!inputWindowRef.value?.contains(target)) {
    closeInput();
  }
};

const addListeners = () => {
  mouseEmitter.on("mousedown", onMouseDown);
  // TODO(Wendy) - come back to this code when we wanna make the input float again
  // window.addEventListener("resize", closeOnResizeOrScroll);
  // window.addEventListener("scroll", closeOnResizeOrScroll, true);
};
const removeListeners = () => {
  mouseEmitter.off("mousedown", onMouseDown);
  // TODO(Wendy) - come back to this code when we wanna make the input float again
  // window.removeEventListener("resize", closeOnResizeOrScroll);
  // window.addEventListener("scroll", closeOnResizeOrScroll, true);
};

const inputOpen = ref(false);
const openInput = () => {
  inputOpen.value = true;
  resetSearch();
  addListeners();
  nextTick(() => {
    if (inputRef.value) {
      inputRef.value.focus();
      inputRef.value.value = "";
    }
  });
};
const closeInput = () => {
  inputOpen.value = false;
  removeListeners();
};

const blur = () => {
  // as long as the input window is open, stay focused on the input!
  inputRef.value?.focus();
};

const possibleConnections = computed(() => {
  const componentIds = Object.keys(ctx.componentDetails.value);
  const alreadyConnectedComponentIds = props.existingEdges.map(
    (edge) => edge.componentId,
  );

  // TODO(Wendy) - maybe instead of just filtering out the components that are already connected
  // we might wanna show them but have some indication that they are already connected?
  return componentIds
    .filter(
      (id) =>
        id !== props.parentComponentId &&
        !alreadyConnectedComponentIds.includes(id),
    )
    .map(
      (id) =>
        ({
          ...ctx.componentDetails.value[id],
          id,
        } as PossibleConnectionComponent),
    );
});

const filteredComponents = reactive<PossibleConnectionComponent[]>([]);

const filterStr = ref<string>("");
const activeFilterStr = ref<string>("");
const debouncedFilterStr = debounce(
  () => {
    activeFilterStr.value = filterStr.value;

    if (!filterStr.value) {
      filteredComponents.splice(0, Infinity);
      return;
    }

    const fzf = new Fzf(possibleConnections.value, {
      casing: "case-insensitive",
      selector: (c) => `${c.name} ${c.schemaVariantName}`,
    });

    const results = fzf.find(filterStr.value);
    const items: PossibleConnectionComponent[] = results.map((fz) => fz.item);
    filteredComponents.splice(0, Infinity, ...items);
  },
  500,
  { trailing: true, leading: false },
);

watch(
  () => filterStr.value,
  () => {
    debouncedFilterStr();
  },
  { immediate: true },
);

const onInputChange = (e: Event) => {
  const v = (e.target as HTMLInputElement).value;

  filterStr.value = v;
};

const selectedOptionIndex = ref(-1);

const onUp = () => {
  selectedOptionIndex.value--;
  if (selectedOptionIndex.value < 0) {
    selectedOptionIndex.value = filteredComponents.length - 1;
  }
};

const onDown = () => {
  selectedOptionIndex.value++;
  if (selectedOptionIndex.value > filteredComponents.length - 1) {
    selectedOptionIndex.value = 0;
  }
};

const resetSearch = () => {
  filteredComponents.splice(0, Infinity);
  filterStr.value = "";
  activeFilterStr.value = "";
  selectedOptionIndex.value = -1;
};

const api = useApi();

const createManagementConnection = (
  component?: PossibleConnectionComponent,
) => {
  let toBeManagedComponentId;
  if (component) {
    toBeManagedComponentId = component.id;
  } else {
    toBeManagedComponentId = filteredComponents[selectedOptionIndex.value]?.id;
  }

  if (!toBeManagedComponentId) return;

  const call = api.endpoint(routes.UpdateComponentManage, {
    id: props.parentComponentId,
  });
  call.post({
    componentId: toBeManagedComponentId,
  } as UpdateComponentManageArgs);
  closeInput();
};
</script>
