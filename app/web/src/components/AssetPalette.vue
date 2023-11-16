<template>
  <div class="inset-0 absolute">
    <template v-if="schemasReqStatus.isPending">
      <div class="w-full p-lg flex flex-col gap-2 items-center">
        <Icon name="loader" size="2xl" />
        <h2>Loading Asset Palette...</h2>
      </div>
    </template>
    <template v-else-if="schemasReqStatus.isSuccess">
      <ScrollArea class="">
        <SiSearch
          autoSearch
          placeholder="search assets"
          @search="onSearchUpdated"
        />
        <template #top>
          <SidebarSubpanelTitle label="Assets" icon="component-plus" />

          <div
            ref="instructionsRef"
            class="border-b-2 dark:border-neutral-600 text-sm leading-tight p-2.5 text-neutral-400 dark:text-neutral-300 flex flex-row items-center gap-2"
          >
            <!-- <a
              href="#"
              class="hover:text-neutral-600 dark:hover:text-neutral-400"
              @click="hideInstructions"
            >
              <Icon name="x-circle" />
            </a> -->
            <div>
              Drag the assets that you wish to include in your application into
              the canvas to the right.
            </div>
          </div>
        </template>

        <ul class="overflow-y-auto">
          <Collapsible
            v-for="(category, categoryIndex) in orderedFilteredComponents"
            :key="categoryIndex"
            :label="category.displayName"
            as="li"
            contentAs="ul"
            defaultOpen
            class="select-none"
          >
            <li
              v-for="(schema, schemaIndex) in category.schemas"
              :key="schemaIndex"
              class="select-none border-b-2 dark:border-neutral-600"
            >
              <SiNodeSprite
                :color="schema.color"
                :name="schema.displayName"
                :class="
                  clsx(
                    'border border-transparent',
                    fixesAreRunning
                      ? 'hover:cursor-progress'
                      : 'hover:border-action-500 dark:hover:border-action-300 dark:text-white hover:text-action-500 dark:hover:text-action-500 hover:cursor-pointer',
                    componentsStore.selectedInsertSchemaId === schema.id
                      ? 'bg-action-100 dark:bg-action-700 border border-action-500 dark:border-action-300'
                      : '',
                  )
                "
                @mousedown.left="onSelect(schema.id, fixesAreRunning)"
                @click.right.prevent
              />
            </li>
          </Collapsible>
        </ul>
      </ScrollArea>
    </template>

    <template v-if="selectedSchema">
      <Teleport to="body">
        <div
          ref="mouseNode"
          class="fixed top-0 pointer-events-none translate-x-[-50%] translate-y-[-50%] z-100"
        >
          <NodeSkeleton :color="selectedSchema.color" />
        </div>
      </Teleport>
    </template>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, onMounted, onBeforeUnmount, ref } from "vue";
import { Collapsible, Icon, ScrollArea } from "@si/vue-lib/design-system";
import clsx from "clsx";
import SiNodeSprite from "@/components/SiNodeSprite.vue";
import { useComponentsStore, MenuSchema } from "@/store/components.store";
import NodeSkeleton from "@/components/NodeSkeleton.vue";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import SiSearch from "@/components/SiSearch.vue";

defineProps<{ fixesAreRunning: boolean }>();

const instructionsRef = ref();

// const hideInstructions = () => {
//   if (instructionsRef.value) {
//     instructionsRef.value.classList.add("hidden");
//   }
// };

const componentsStore = useComponentsStore();
// NOTE - component store is automatically fetching things we need when it is used
// otherwise we could trigger calls here

const schemasReqStatus = componentsStore.getRequestStatus(
  "FETCH_AVAILABLE_SCHEMAS",
);

const filterString = ref("");
const filterStringCleaned = computed(() =>
  filterString.value.trim().toLowerCase(),
);
const filterModeActive = computed(() => !!filterStringCleaned.value);

function onSearchUpdated(newFilterString: string) {
  filterString.value = newFilterString;
}
const addMenuData = computed(() => componentsStore.nodeAddMenu);

// TODO(nick): fold this into "filteredComponents" so that they are already sorted.
// For context, TypeScript was super rusty (pun not intended) when I wrote this.
const orderedFilteredComponents = computed(() => {
  const orderedFilteredComponents = [];
  for (const filteredComponent of filteredComponents.value) {
    const schemas = filteredComponent.schemas;
    filteredComponent.schemas = _.orderBy(schemas, "displayName");
    orderedFilteredComponents.push(filteredComponent);
  }
  return _.orderBy(orderedFilteredComponents, "displayName");
});

const filteredComponents = computed(() => {
  if (!filterModeActive.value) return addMenuData.value;
  // need a deep clone because of the complex object
  return _.filter(_.cloneDeep(addMenuData.value), (c) => {
    // if the string matches the group, return the whole thing
    if (c.displayName.toLowerCase().includes(filterStringCleaned.value))
      return true;
    // otherwise, filter out the individual assets that don't match
    c.schemas = _.filter(c.schemas, (s) =>
      s.displayName.toLowerCase().includes(filterStringCleaned.value),
    );
    return c.schemas.length > 0;
  });
});

const schemasById = computed(() => {
  return addMenuData.value.reduce((p, c) => {
    c.schemas.forEach((schema) => {
      p[schema.id] = schema;
    });
    return p;
  }, {} as Record<string, MenuSchema>);
});
const selectedSchema = computed(() => {
  if (componentsStore.selectedInsertSchemaId)
    return schemasById.value[componentsStore.selectedInsertSchemaId];
  return undefined;
});
const selecting = ref(false);
const mouseNode = ref();

const updateMouseNode = (e: MouseEvent) => {
  if (mouseNode.value) {
    const mouseX = e.clientX;
    const mouseY = e.clientY;
    mouseNode.value.style.left = `${mouseX}px`;
    mouseNode.value.style.top = `${mouseY}px`;
  }
};

function onSelect(schemaId: string, fixesAreRunning: boolean) {
  if (fixesAreRunning) {
    // Prevent selection while fixes are running
    return;
  }

  componentsStore.selectedInsertSchemaId = schemaId;
  selecting.value = true;
}

function onDeselect() {
  componentsStore.selectedInsertSchemaId = null;
}

const onKeyDown = (e: KeyboardEvent) => {
  if (e.key === "Escape" || e.key === "Backspace") {
    onDeselect();
  }
};

const onMouseDown = (e: MouseEvent) => {
  updateMouseNode(e);
  if (selecting.value) selecting.value = false;
  else onDeselect();
};

const onMouseMove = (e: MouseEvent) => {
  updateMouseNode(e);
};

onMounted(() => {
  window.addEventListener("mousemove", onMouseMove);
  window.addEventListener("keydown", onKeyDown);
  window.addEventListener("mousedown", onMouseDown);
});

onBeforeUnmount(() => {
  window.removeEventListener("mousemove", onMouseMove);
  window.removeEventListener("keydown", onKeyDown);
  window.removeEventListener("mousedown", onMouseDown);
});
</script>
