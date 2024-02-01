<template>
  <div class="inset-0 absolute">
    <template v-if="schemasReqStatus.isPending || addMenuReqStatus.isPending">
      <div class="w-full p-lg flex flex-col gap-2 items-center">
        <Icon name="loader" size="2xl" />
        <h2>Loading Asset Palette...</h2>
      </div>
    </template>
    <template v-else-if="schemasReqStatus.isSuccess">
      <ScrollArea>
        <template #top>
          <SidebarSubpanelTitle label="Assets" icon="component-plus">
            <Icon
              v-tooltip="{
                content:
                  'Drag the assets that you wish to include in your application into the canvas to the right.',
                theme: 'w-380',
              }"
              class="cursor-pointer"
              name="question-circle"
            />
          </SidebarSubpanelTitle>

          <SiSearch
            autoSearch
            placeholder="search assets"
            @search="onSearchUpdated"
          />
        </template>

        <ul class="overflow-y-auto">
          <Collapsible
            v-for="(category, categoryIndex) in filteredComponents"
            ref="collapsibleRefs"
            :key="categoryIndex"
            :label="category.displayName"
            as="li"
            contentAs="ul"
            class="select-none"
          >
            <li
              v-for="(schema, schemaIndex) in category.schemas"
              :key="schemaIndex"
              class="select-none border-b-2 dark:border-neutral-600"
              data-cy="asset_card"
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
                @mousedown.left.stop="onSelect(schema.id, fixesAreRunning)"
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
import { windowListenerManager } from "@si/vue-lib";
import SiNodeSprite from "@/components/SiNodeSprite.vue";
import {
  useComponentsStore,
  MenuSchema,
  NodeAddMenu,
} from "@/store/components.store";
import NodeSkeleton from "@/components/NodeSkeleton.vue";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import SiSearch from "@/components/SiSearch.vue";

defineProps<{ fixesAreRunning: boolean }>();

const componentsStore = useComponentsStore();
// NOTE - component store is automatically fetching things we need when it is used
// otherwise we could trigger calls here

// TODO - probably should not need 2 requests here. currently we only use schema variants for the colors...
const schemasReqStatus = componentsStore.getRequestStatus(
  "FETCH_AVAILABLE_SCHEMAS",
);
const addMenuReqStatus = componentsStore.getRequestStatus(
  "FETCH_NODE_ADD_MENU",
);

const collapsibleRefs = ref<InstanceType<typeof Collapsible>[]>([]);

const filterString = ref("");
const filterStringCleaned = computed(() =>
  filterString.value.trim().toLowerCase(),
);
const filterModeActive = computed(() => !!filterStringCleaned.value);

function onSearchUpdated(newFilterString: string) {
  filterString.value = newFilterString;
  collapsibleRefs.value.forEach((c) => {
    c.toggleIsOpen(true);
  });
}
const addMenuData = computed(() => componentsStore.nodeAddMenu);

const filteredComponents = computed(() => {
  if (!filterModeActive.value) return addMenuData.value;

  const filteredCategories = [] as NodeAddMenu;
  _.each(addMenuData.value, (c) => {
    // if the string matches the group, add the whole thing
    if (c.displayName.toLowerCase().includes(filterStringCleaned.value)) {
      filteredCategories.push(c);
      return;
    }

    // otherwise, filter out the individual assets that don't match
    const matchingSchemas = _.filter(c.schemas, (s) => {
      const categoryAndSchemaName = `${c.displayName} ${s.displayName}`;
      return categoryAndSchemaName
        .toLowerCase()
        .includes(filterStringCleaned.value);
    });

    if (matchingSchemas.length > 0) {
      filteredCategories.push({
        displayName: c.displayName,
        schemas: matchingSchemas,
      });
    }
  });
  return filteredCategories;
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

  if (componentsStore.selectedInsertSchemaId === schemaId) {
    componentsStore.cancelInsert();
  } else {
    componentsStore.setInsertSchema(schemaId);
  }
}

const onKeyDown = (e: KeyboardEvent) => {
  if (
    (e.key === "Escape" || e.key === "Backspace") &&
    componentsStore.selectedInsertSchemaId
  ) {
    componentsStore.cancelInsert();
    e.stopPropagation();
  }
};

const onMouseDown = (e: MouseEvent) => {
  updateMouseNode(e);
  if (componentsStore.selectedInsertSchemaId) {
    componentsStore.cancelInsert();
  }
};

const onMouseMove = (e: MouseEvent) => {
  updateMouseNode(e);
};

onMounted(() => {
  windowListenerManager.addEventListener("mousemove", onMouseMove);
  windowListenerManager.addEventListener("keydown", onKeyDown, 5);
  windowListenerManager.addEventListener("mousedown", onMouseDown);
});

onBeforeUnmount(() => {
  windowListenerManager.removeEventListener("mousemove", onMouseMove);
  windowListenerManager.removeEventListener("keydown", onKeyDown);
  windowListenerManager.removeEventListener("mousedown", onMouseDown);
});
</script>
