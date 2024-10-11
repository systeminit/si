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
                <div id="asset-palette-title">Assets</div>
                <PillCounter :count="assetCount" />
              </div>
            </template>
            <div class="flex flex-row items-center gap-xs">
              <Icon
                v-tooltip="{
                  content:
                    'Drag the assets that you wish to include in your application on to the canvas to the right.',
                  theme: 'w-380',
                }"
                class="cursor-pointer hover:text-shade-100 dark:hover:text-shade-0"
                name="question-circle"
              />
            </div>
          </SidebarSubpanelTitle>

          <SiSearch
            ref="searchRef"
            placeholder="search assets"
            :filters="searchFiltersWithCounts"
            @search="onSearchUpdated"
          />
        </template>

        <TreeNode
          v-for="category in filteredCategoriesAndSchemas"
          ref="collapsibleRefs"
          :key="category.displayName"
          :label="category.displayName"
          :primaryIcon="getAssetIcon(category.displayName)"
          :color="category.schemaVariants[0]?.variant.color || '#000'"
          enableDefaultHoverClasses
          enableGroupToggle
          alwaysShowArrow
          indentationSize="none"
        >
          <template #icons>
            <PillCounter
              :count="category.schemaVariants.length"
              showHoverInsideTreeNode
            />
          </template>
          <TreeNode
            v-for="schemaVariant in category.schemaVariants"
            :key="
              schemaVariant.type === 'installed'
                ? schemaVariant.variant.schemaVariantId
                : schemaVariant.variant.schemaId
            "
            :color="schemaVariant.variant.color"
            :classes="
              clsx(
                'dark:text-white text-black dark:bg-neutral-800 py-[1px]',
                'hover:dark:outline-action-300 hover:outline-action-500 hover:outline hover:z-10 hover:-outline-offset-1 hover:outline-1',
              )
            "
            :isSelected="
              componentsStore.selectedInsertCategoryVariantId ===
              schemaVariant.id
            "
            showSelection
            @mousedown.left.stop="onSelect(schemaVariant.id, $event)"
            @click.right.prevent
          >
            <template #label>
              <TruncateWithTooltip class="text-xs">
                {{
                  schemaVariantDisplayName(
                    schemaVariant.variant as SchemaVariant,
                  )
                }}
              </TruncateWithTooltip>
              <!-- <div
                class="italic text-xs text-neutral-500 dark:text-neutral-400"
              >
                asset by: System Initiative
              </div> -->
            </template>
            <template #icons>
              <EditingPill
                v-if="
                  schemaVariant.type === 'installed' &&
                  !schemaVariant.variant.isLocked
                "
                :color="schemaVariant.variant.color"
              />
            </template>
          </TreeNode>
        </TreeNode>
      </ScrollArea>
    </template>

    <template v-if="selectedSchemaVariant">
      <Teleport to="body">
        <div
          ref="mouseNode"
          class="fixed top-0 pointer-events-none translate-x-[-50%] translate-y-[-50%] z-100"
        >
          <NodeSkeleton :color="selectedSchemaVariant.variant.color" />
        </div>
      </Teleport>
    </template>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, onMounted, onBeforeUnmount, ref, nextTick } from "vue";
import {
  Icon,
  PillCounter,
  ScrollArea,
  TreeNode,
  SiSearch,
  Filter,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { windowListenerManager } from "@si/vue-lib";
import {
  useComponentsStore,
  getAssetIcon,
  Categories,
} from "@/store/components.store";
import { schemaVariantDisplayName } from "@/store/asset.store";
import { SchemaVariant } from "@/api/sdf/dal/schema";
import NodeSkeleton from "@/components/NodeSkeleton.vue";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import EditingPill from "@/components/EditingPill.vue";

const searchRef = ref<InstanceType<typeof SiSearch>>();

const componentsStore = useComponentsStore();

const schemasReqStatus = componentsStore.getRequestStatus(
  "FETCH_AVAILABLE_SCHEMAS",
);

const collapsibleRefs = ref<InstanceType<typeof TreeNode>[]>([]);

const searchString = ref("");
const searchStringCleaned = computed(() =>
  searchString.value.trim().toLowerCase(),
);
const filterModeActive = computed(
  () => !!(searchStringCleaned.value || searchRef.value?.filteringActive),
);

function onSearchUpdated(newFilterString: string) {
  searchString.value = newFilterString;
  collapsibleRefs.value.forEach((c) => {
    c.toggleIsOpen(true);
  });
}
const categories = computed(() => componentsStore.categories);

const filteredCategoriesBySearchString = (
  categories: Categories,
  searchString = searchStringCleaned.value,
) => {
  const inProgress = [] as Categories;
  _.each(categories, (c) => {
    // if the string matches the group, add the whole thing
    if (c.displayName.toLowerCase().includes(searchString)) {
      inProgress.push(c);
      return;
    }

    // otherwise, filter out the individual assets that don't match
    const matchingSchemas = _.filter(c.schemaVariants, (s) => {
      const categoryAndSchemaName = `${c.displayName} ${s.variant.schemaName}`;
      return categoryAndSchemaName.toLowerCase().includes(searchString);
    });

    if (matchingSchemas.length > 0) {
      inProgress.push({
        displayName: c.displayName,
        schemaVariants: matchingSchemas,
      });
    }
  });
  return inProgress;
};

const filteredCategoriesBySearchStringAndFilters = (
  categories: Categories,
  searchString = searchStringCleaned.value,
) => {
  let filteredAssets = filteredCategoriesBySearchString(
    categories,
    searchString,
  );

  if (searchRef.value?.filteringActive) {
    for (
      let index = 0;
      index < searchRef.value?.activeFilters.length;
      index++
    ) {
      if (searchRef.value?.activeFilters[index]) {
        const filterFunction = filterFunctions[index];
        if (filterFunction) {
          filteredAssets = filterFunction(filteredAssets);
        }
      }
    }
  }

  return filteredAssets;
};

const filteredCategoriesAndSchemas = computed(() => {
  if (!filterModeActive.value) return categories.value;
  else return filteredCategoriesBySearchStringAndFilters(categories.value);
});

const searchFiltersWithCounts = computed(() => {
  const searchFilters: Array<Filter> = [
    {
      name: "AWS",
      iconColor: "#FF9900",
      iconName: "logo-aws",
      count: computed(() =>
        getAssetCount(
          filteredCategoriesBySearchStringAndFilters(
            filteredCategoriesBySearchStringAndFilters(categories.value),
            "aws",
          ),
        ),
      ).value,
    },
    {
      name: "Docker",
      iconColor: "#4695e7",
      iconName: "logo-docker",
      count: computed(() =>
        getAssetCount(
          filteredCategoriesBySearchStringAndFilters(
            filteredCategoriesBySearchStringAndFilters(categories.value),
            "docker",
          ),
        ),
      ).value,
    },
  ];

  return searchFilters;
});

const filterFunctions = [
  // AWS FILTER
  (assets: Categories) => {
    return filteredCategoriesBySearchString(assets, "aws");
  },
  // DOCKER FILTER
  (assets: Categories) => {
    return filteredCategoriesBySearchString(assets, "docker");
  },
];

const getAssetCount = (categories: Categories) => {
  let count = 0;

  categories.forEach((category) => {
    count += category.schemaVariants.length;
  });

  return count;
};

const assetCount = computed(() =>
  getAssetCount(filteredCategoriesAndSchemas.value),
);

const selectedSchemaVariant = computed(() => {
  if (componentsStore.selectedInsertCategoryVariantId)
    return componentsStore.categoryVariantById[
      componentsStore.selectedInsertCategoryVariantId
    ];
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

function onSelect(id: string, e: MouseEvent) {
  if (componentsStore.selectedInsertCategoryVariantId === id) {
    componentsStore.cancelInsert();
  } else {
    componentsStore.setInsertSchema(id);
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
    componentsStore.selectedInsertCategoryVariantId
  ) {
    componentsStore.cancelInsert();
    e.stopPropagation();
  }
};

const onMouseDown = (e: MouseEvent) => {
  updateMouseNode(e);
  if (componentsStore.selectedInsertCategoryVariantId) {
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
