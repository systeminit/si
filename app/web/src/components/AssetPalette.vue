<template>
  <div class="inset-0 absolute asset-palette">
    <template v-if="schemasReqStatus.isPending">
      <div class="w-full p-lg flex flex-col gap-xs items-center">
        <Icon name="loader" size="2xl" />
        <h2>Loading Asset Palette...</h2>
      </div>
    </template>
    <template v-else-if="schemasReqStatus.isSuccess">
      <ScrollArea>
        <template #top>
          <SidebarSubpanelTitle icon="component-plus">
            <template #label>
              <div class="flex flex-row gap-xs">
                <div>Assets</div>
                <PillCounter :count="assetCount" borderTone="action" />
              </div>
            </template>
            <div class="flex flex-row items-center gap-xs">
              <Icon
                v-tooltip="{
                  content:
                    'Drag the assets that you wish to include in your application into the canvas to the right.',
                  theme: 'w-380',
                }"
                class="cursor-pointer hover:text-shade-100 dark:hover:text-shade-0"
                name="question-circle"
              />
            </div>
          </SidebarSubpanelTitle>

          <SiSearch
            autoSearch
            placeholder="search assets"
            @search="onSearchUpdated"
          />
        </template>

        <TreeNode
          v-for="(category, categoryIndex) in filteredCategoriesAndSchemas"
          :key="categoryIndex"
          :label="category.displayName"
          :primaryIcon="getAssetIcon(category.displayName)"
          :color="category.schemas[0]?.color || '#000'"
          classes="bg-neutral-100 dark:bg-neutral-700 group/tree"
          labelClasses="font-bold select-none hover:text-action-500 dark:hover:text-action-300"
          enableGroupToggle
          alwaysShowArrow
          clickLabelToToggle
          indentationSize="none"
        >
          <template #icons>
            <PillCounter
              :count="category.schemas.length"
              borderTone="action"
              class="group-hover/tree:text-action-500 dark:group-hover/tree:text-action-300 group-hover/tree:bg-action-100 dark:group-hover/tree:bg-action-800"
            />
          </template>
          <TreeNode
            v-for="(schema, schemaIndex) in category.schemas"
            :key="schemaIndex"
            :color="schema.color"
            :classes="
              clsx(
                'dark:text-white text-black dark:bg-neutral-800 py-[1px]',
                'hover:dark:outline-action-300 hover:outline-action-500 hover:outline hover:z-10 hover:-outline-offset-1 hover:outline-1',
                componentsStore.selectedInsertSchemaId === schema.id
                  ? 'bg-action-100 dark:bg-action-700 border border-action-500 dark:border-action-300 py-0'
                  : 'dark:hover:text-action-300 hover:text-action-500',
              )
            "
            :isSelected="componentsStore.selectedInsertSchemaId === schema.id"
            @mousedown.left.stop="onSelect(schema.id, $event)"
            @click.right.prevent
          >
            <template #label>
              <div class="text-sm">
                {{ schema.name }}
              </div>
              <!-- <div
                class="italic text-xs text-neutral-500 dark:text-neutral-400"
              >
                asset by: System Initiative
              </div> -->
            </template>
          </TreeNode>
        </TreeNode>
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
import { computed, onMounted, onBeforeUnmount, ref, nextTick } from "vue";
import {
  Collapsible,
  Icon,
  PillCounter,
  ScrollArea,
  TreeNode,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { windowListenerManager } from "@si/vue-lib";
import {
  useComponentsStore,
  getAssetIcon,
  Categories,
  DiagramSchemaWithDisplayMetadata,
} from "@/store/components.store";
import NodeSkeleton from "@/components/NodeSkeleton.vue";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import SiSearch from "@/components/SiSearch.vue";

const componentsStore = useComponentsStore();

const schemasReqStatus = componentsStore.getRequestStatus(
  "FETCH_AVAILABLE_SCHEMAS",
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
const categories = computed(() => componentsStore.categories);

const filteredCategoriesAndSchemas = computed(() => {
  if (!filterModeActive.value) return categories.value;

  const inProgress = [] as Categories;
  _.each(categories.value, (c) => {
    // if the string matches the group, add the whole thing
    if (c.displayName.toLowerCase().includes(filterStringCleaned.value)) {
      inProgress.push(c);
      return;
    }

    // otherwise, filter out the individual assets that don't match
    const matchingSchemas = _.filter(c.schemas, (s) => {
      const categoryAndSchemaName = `${c.displayName} ${s.name}`;
      return categoryAndSchemaName
        .toLowerCase()
        .includes(filterStringCleaned.value);
    });

    if (matchingSchemas.length > 0) {
      inProgress.push({
        displayName: c.displayName,
        schemas: matchingSchemas,
      });
    }
  });
  return inProgress;
});

const assetCount = computed(() => {
  let count = 0;

  filteredCategoriesAndSchemas.value.forEach((category) => {
    count += category.schemas.length;
  });

  return count;
});

const schemasById = computed(() => {
  return categories.value.reduce((p, c) => {
    c.schemas.forEach((schema) => {
      p[schema.id] = schema;
    });
    return p;
  }, {} as Record<string, DiagramSchemaWithDisplayMetadata>);
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

function onSelect(schemaId: string, e: MouseEvent) {
  if (componentsStore.selectedInsertSchemaId === schemaId) {
    componentsStore.cancelInsert();
  } else {
    componentsStore.setInsertSchema(schemaId);
    if (e) {
      nextTick(() => {
        updateMouseNode(e);
      });
    }
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
