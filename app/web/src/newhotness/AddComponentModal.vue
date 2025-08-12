<template>
  <!-- NOTE: the Modal CSS for height in "max" doesn't work as we might expect -->
  <Modal
    ref="modalRef"
    noWrapper
    hideExitButton
    size="max"
    class="w-[max(800px,66vw)]"
    @click="onClick"
  >
    <div class="h-[45svh]">
      <div
        :class="
          clsx(
            'grid createcomponent gap-xs [&>*]:border h-full',
            selectedAsset && 'grid grid-cols-2',
            themeClasses(
              '[&>*]:bg-shade-0 border-neutral-400 [&_*]:border-neutral-400',
              '[&>*]:bg-neutral-900 border-neutral-600 [&_*]:border-neutral-600',
            ),
          )
        "
      >
        <!-- Left side - search, filters, asset list -->
        <div ref="leftSideRef" class="assets flex flex-col gap-xs">
          <header class="text-md border-b p-xs flex-none">
            Create a component
          </header>

          <!-- I don't like that we have to specify a height here it should
          contain within its parent, and its possible, just dont want to spend more time on it -->
          <div class="flex flex-col gap-xs grow p-xs max-h-[41svh]">
            <!-- Fuzzy search input, is focused when the Modal is opened-->
            <SiSearch
              ref="searchRef"
              v-model="fuzzySearchString"
              data-testid="add-component-search"
              placeholder="Start typing to filter components"
              :borderBottom="false"
              class="flex-none border"
              @blur="searchRef?.focusSearch()"
              @input="clearSelection"
              @keydown.enter.prevent="onEnter"
              @keydown.up.prevent="onUp"
              @keydown.down.prevent="onDown"
              @keydown.left.prevent="onLeft"
              @keydown.right.prevent="onRight"
              @keydown.tab.prevent="onTab"
            >
              <template #right>
                <div
                  v-if="showResults"
                  :class="
                    clsx(
                      'flex flex-row flex-none gap-3xs items-center text-2xs pr-2xs',
                      themeClasses('text-shade-100', 'text-shade-0'),
                    )
                  "
                >
                  <TextPill tighter variant="key2">Up</TextPill>
                  <TextPill tighter variant="key2">Down</TextPill>
                  <div class="leading-snug">to navigate</div>
                </div>
              </template>
            </SiSearch>
            <!-- Fuzzy search category filters -->
            <HorizontalScrollArea
              hideScrollbar
              class="flex flex-row gap-xs flex-none"
            >
              <FilterTile
                v-for="filter in componentFilters"
                :key="filter.name"
                :label="filter.name"
                :count="filter.count"
                :color="filter.color"
                :icon="filter.icon"
                :selected="isFilterSelected(filter.name)"
                @click="toggleFilterTile(filter.name)"
              />
            </HorizontalScrollArea>
            <!-- Fuzzy search results list -->
            <div v-if="showResults" class="grow min-h-0 scrollable">
              <TreeNode
                v-for="category in filteredCategories"
                ref="categoryTreeNodeRefs"
                :key="category.name"
                :defaultOpen="
                  !(
                    debouncedSearchString.length === 0 &&
                    selectedFilter === undefined
                  )
                "
                :class="themeClasses('bg-neutral-200', 'bg-neutral-700')"
                indentationSize="none"
                :label="category.name"
                alwaysShowArrow
                clickLabelToToggle
                enableGroupToggle
                :primaryIcon="category.icon"
                :color="category.color"
              >
                <TreeNode
                  v-for="asset in category.assets"
                  :key="asset.key.schemaId + asset.key.schemaVariantId"
                  :class="
                    clsx(
                      'hover:outline hover:z-10 hover:-outline-offset-1 hover:outline-1',
                      themeClasses(
                        'bg-shade-0 hover:outline-action-500',
                        'bg-neutral-800 hover:outline-action-300',
                      ),
                      compareKeys(selectedAsset?.key, asset.key) && [
                        'add-component-selected-item',
                        themeClasses(
                          'outline-action-500 bg-action-200',
                          'outline-action-300 bg-action-900',
                        ),
                      ],
                    )
                  "
                  :color="asset.variant.color"
                  @click="() => selectAsset(asset)"
                >
                  <template #label>
                    <!-- TODO(Wendy) - style this text based on the fuzzy search! -->
                    <div class="flex flex-row items-center gap-xs">
                      <TruncateWithTooltip>
                        {{ asset.name }}
                      </TruncateWithTooltip>
                      <EditingPill
                        v-if="!asset.variant.isLocked"
                        :color="asset.variant.color"
                      />
                    </div>
                  </template>
                  <template
                    v-if="compareKeys(selectedAsset?.key, asset.key)"
                    #icons
                  >
                    <Icon v-if="api.inFlight.value" name="loader" size="sm" />
                    <div
                      v-else
                      :class="
                        clsx(
                          'text-xs',
                          themeClasses('text-neutral-900', 'text-neutral-200'),
                        )
                      "
                    >
                      <TextPill tighter variant="key2">Enter</TextPill> to add
                    </div>
                  </template>
                </TreeNode>
              </TreeNode>
              <EmptyState
                v-if="filteredCategories.length === 0"
                text="No Components Found"
                secondaryText="Your search parameters did not match any components"
                icon="alert-circle"
              />
            </div>
          </div>
        </div>
        <!-- Right side - documentation -->
        <template v-if="selectedAsset">
          <div
            :class="
              clsx(
                'docs border overflow-hidden break-words',
                'flex flex-col flex-1 pb-xs gap-2xs',
              )
            "
          >
            <TruncateWithTooltip
              :class="clsx('text-lg font-bold flex-none px-xs py-2xs border-b')"
            >
              {{ selectedAsset.name }}
            </TruncateWithTooltip>
            <div
              class="flex flex-col gap-2xs px-xs scrollable flex-1 min-h-0 [&>.markdown_*]:text-sm"
            >
              <a
                v-if="selectedAsset.variant.link"
                target="_blank"
                :href="selectedAsset.variant.link"
                :class="
                  clsx(
                    'flex-none italic hover:underline',
                    themeClasses('text-action-500', 'text-action-300'),
                  )
                "
              >
                {{ selectedAsset.variant.link }}
              </a>
              <p v-if="selectedAsset.variant.description" class="markdown">
                <MarkdownRender
                  :source="selectedAsset.variant.description"
                  removeMargins
                />
              </p>
              <div
                v-if="
                  !selectedAsset.variant.link &&
                  !selectedAsset.variant.description
                "
                class="h-full flex flex-row items-center justify-center pb-lg"
              >
                <EmptyState icon="docs" text="No Documentation Available" />
              </div>
            </div>
          </div>
        </template>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import {
  BRAND_COLOR_FILTER_HEX_CODES,
  HorizontalScrollArea,
  Icon,
  IconNames,
  Modal,
  SiSearch,
  themeClasses,
  TreeNode,
  TruncateWithTooltip,
  TextPill,
} from "@si/vue-lib/design-system";
import { computed, inject, nextTick, ref, watch } from "vue";
import clsx from "clsx";
import { useQuery, useQueryClient } from "@tanstack/vue-query";
import { useRoute, useRouter } from "vue-router";
import { debounce } from "lodash-es";
import { FzfResultItem } from "fzf";
import EditingPill from "@/components/EditingPill.vue";
import {
  BifrostSchemaVariantCategories,
  CategoryVariant,
  EntityKind,
  SchemaVariant,
  UninstalledVariant,
  BifrostComponent,
  EddaComponent,
  AttributeTree,
} from "@/workers/types/entity_kind_types";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import { useFzf } from "./logic_composables/fzf";
import FilterTile from "./layout_components/FilterTile.vue";
import { assertIsDefined, Context, ExploreContext } from "./types";
import { componentTypes, routes, useApi } from "./api_composables";
import EmptyState from "./EmptyState.vue";
import MarkdownRender from "./MarkdownRender.vue";

const ctx: Context | undefined = inject("CONTEXT");
assertIsDefined(ctx);
const bannerClosed = ref(false);

const selectedAsset = ref<UIAsset | undefined>(undefined);
const selectAsset = (asset: UIAsset) => {
  if (compareKeys(selectedAsset.value?.key, asset.key)) onEnter();
  else selectedAsset.value = asset;
  selectionIndex.value = filteredAssetsFlat.value.findIndex((a) =>
    compareKeys(a.key, asset.key),
  );

  // scroll selected item into view
  nextTick(() => {
    const el = document.getElementsByClassName(
      "add-component-selected-item",
    )[0];
    if (el) {
      el.scrollIntoView({ block: "center" });
    }
  });
};
const clearSelection = () => {
  selectedAsset.value = undefined;
  selectionIndex.value = undefined;
};

const explore = inject<ExploreContext>("EXPLORE_CONTEXT");
assertIsDefined<ExploreContext>(explore);

const selectionIndex = ref<number | undefined>();

const route = useRoute();
const router = useRouter();
const api = useApi();
const queryClient = useQueryClient();
const makeKey = useMakeKey();
const makeArgs = useMakeArgs();

const onEnter = async () => {
  if (api.inFlight.value) return; // you've already submitted, disable submission

  if (!selectedAsset.value) return;
  const variant =
    "uninstalled" in selectedAsset.value.variant
      ? (selectedAsset.value.variant as UninstalledVariant)
      : (selectedAsset.value.variant as SchemaVariant);
  let params: componentTypes.ComponentIdType;
  if ("schemaVariantId" in variant)
    params = {
      schemaType: "installed",
      schemaVariantId: variant.schemaVariantId,
    };
  else
    params = {
      schemaType: "uninstalled",
      schemaId: variant.schemaId,
    };

  const payload = componentTypes.createComponentPayload(params);
  const call = api.endpoint<{
    componentId: string;
    materializedView: EddaComponent;
    attributeTreeMaterializedView: AttributeTree;
    schemaVariantMaterializedView: SchemaVariant;
  }>(routes.CreateComponent, {
    viewId: explore.viewId.value,
  });

  const { req, newChangeSetId } =
    await call.post<componentTypes.CreateComponentPayload>(payload);

  if (api.ok(req)) {
    // Set the new component and attribute tree in the query cache
    const componentQueryKey = makeKey(
      EntityKind.Component,
      req.data.componentId,
    ).value;
    const attributeTreeQueryKey = makeKey(
      EntityKind.AttributeTree,
      req.data.componentId,
    ).value;
    // replace old change set id with new one for the query key
    if (newChangeSetId) {
      componentQueryKey.forEach((v, idx) => {
        if (v === route.params.changeSetId)
          componentQueryKey[idx] = newChangeSetId;
      });
      attributeTreeQueryKey.forEach((v, idx) => {
        if (v === route.params.changeSetId)
          attributeTreeQueryKey[idx] = newChangeSetId;
      });
    }

    const eddaComponent = req.data.materializedView;
    const bifrostComponent: BifrostComponent = {
      ...eddaComponent,
      // This can be set as false as the system doesn't really care about it in this menu
      canBeUpgraded: false,
      schemaVariant: req.data.schemaVariantMaterializedView,
    };
    queryClient.setQueryData(componentQueryKey, bifrostComponent);

    queryClient.setQueryData(
      attributeTreeQueryKey,
      req.data.attributeTreeMaterializedView,
    );

    const to = {
      name: "new-hotness-component",
      params: {
        workspacePk: route.params.workspacePk,
        changeSetId: newChangeSetId || route.params.changeSetId,
        componentId: req.data.componentId,
      },
    };
    if (newChangeSetId) api.navigateToNewChangeSet(to, newChangeSetId);
    else router.push(to);
  }
};
const onUp = (e: KeyboardEvent) => {
  if (!showResults.value) return;

  const goByCategory =
    e.key !== "Tab" && (e.shiftKey || e.ctrlKey || e.metaKey);

  if (selectionIndex.value === undefined) {
    selectionIndex.value = filteredAssetsFlat.value.length - 1;
  } else if (goByCategory) {
    selectFirstInNextCategory(selectionIndex.value, -1);
    return;
  } else {
    selectionIndex.value--;
    if (selectionIndex.value < 0) {
      selectionIndex.value = filteredAssetsFlat.value.length - 1;
    }
  }
  selectAssetByIndex();
};
const onDown = (e: KeyboardEvent) => {
  if (!showResults.value) return;

  const goByCategory =
    e.key !== "Tab" && (e.shiftKey || e.ctrlKey || e.metaKey);

  if (selectionIndex.value === undefined) {
    selectionIndex.value = 0;
  } else if (goByCategory) {
    selectFirstInNextCategory(selectionIndex.value, 1);
    return;
  } else {
    selectionIndex.value++;
    if (selectionIndex.value > filteredAssetsFlat.value.length - 1) {
      selectionIndex.value = 0;
    }
  }
  selectAssetByIndex();
};
const onLeft = () => {
  let currentFilterIndex = componentFilters.value.findIndex(
    (filter) => filter.name === selectedFilter.value,
  );

  if (currentFilterIndex < 1) {
    selectedFilter.value = undefined;
  } else {
    currentFilterIndex--;
    const newFilter = componentFilters.value[currentFilterIndex];
    if (newFilter) {
      selectedFilter.value = newFilter.name;
    }
  }
};
const onRight = () => {
  let currentFilterIndex = componentFilters.value.findIndex(
    (filter) => filter.name === selectedFilter.value,
  );

  if (currentFilterIndex === componentFilters.value.length - 1) {
    return;
  } else {
    if (currentFilterIndex < 1) currentFilterIndex = 1;
    else currentFilterIndex++;
    const newFilter = componentFilters.value[currentFilterIndex];
    if (newFilter) {
      selectedFilter.value = newFilter.name;
    }
  }
};
const onTab = (e: KeyboardEvent) => {
  if (!showResults.value) return;

  if (e.shiftKey) onUp(e);
  else onDown(e);
};
const selectAssetByIndex = () => {
  if (
    selectionIndex.value !== undefined &&
    filteredAssetsFlat.value[selectionIndex.value]
  ) {
    selectAsset(filteredAssetsFlat.value[selectionIndex.value] as UIAsset);
  }
};
const selectFirstInNextCategory = (currentIndex: number, direction: 1 | -1) => {
  const currentSelection = filteredAssetsFlat.value[currentIndex];
  if (!currentSelection) return;
  const currentCategoryIndex = filteredCategories.value.findIndex((category) =>
    category.assets.find((asset) =>
      compareKeys(asset.key, currentSelection.key),
    ),
  );
  if (currentCategoryIndex > -1) {
    const nextCategory =
      filteredCategories.value[currentCategoryIndex + direction];
    const firstCategoryAsset = filteredCategories.value[0]?.assets[0];
    const lastCategoryAsset =
      filteredCategories.value[filteredCategories.value.length - 1]?.assets[0];
    if (nextCategory && nextCategory.assets[0]) {
      selectAsset(nextCategory.assets[0]);
    } else if (direction === 1 && firstCategoryAsset) {
      selectAsset(firstCategoryAsset);
    } else if (direction === -1 && lastCategoryAsset) {
      selectAsset(lastCategoryAsset);
    }
  }
};

export type AssetFilter = {
  name: string;
  icon: IconNames;
  count: number;
  color?: string;
};

type UICategoryInfo = {
  name: string;
  icon?: IconNames;
  color: string;
};

type UICategory = UICategoryInfo & {
  assets: UIAsset[];
};

type UIAsset = {
  key: UISchemaKey;
  name: string;
  variant: CategoryVariant;
  uiCategory: UICategoryInfo;
};

type UISchemaKey = {
  schemaId: string;
  schemaVariantId?: string;
};

const queryKey = makeKey(EntityKind.SchemaVariantCategories);
const schemaVariantCategoriesOverBifrost =
  useQuery<BifrostSchemaVariantCategories | null>({
    queryKey,
    queryFn: async () =>
      await bifrost<BifrostSchemaVariantCategories>(
        makeArgs(EntityKind.SchemaVariantCategories),
      ),
  });

const categories = computed(() => {
  const rawCategoryData =
    schemaVariantCategoriesOverBifrost.data.value?.categories ?? [];
  return rawCategoryData.map((rawCategory) => {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const firstSV = rawCategory.schemaVariants[0]!;
    const categoryInfo: UICategoryInfo = {
      name: rawCategory.displayName,
      color: firstSV.color,
      icon: pickIcon(rawCategory.displayName),
    };
    const category: UICategory = {
      ...categoryInfo,
      assets: [],
    };
    rawCategory.schemaVariants.forEach((sv) => {
      const asset: UIAsset = {
        key: buildKey(sv),
        variant: sv,
        name: sv.displayName ?? sv.schemaName,
        uiCategory: categoryInfo,
      };
      category.assets.push(asset);
    });
    return category;
  });
});

const compareKeys = (
  key1: UISchemaKey | undefined,
  key2: UISchemaKey | undefined,
) => {
  if (!key1 || !key2) return false;
  return (
    key1.schemaId === key2.schemaId &&
    key1.schemaVariantId === key2.schemaVariantId
  );
};

const buildKey = (sv: CategoryVariant) => {
  const variant =
    "uninstalled" in sv ? (sv as UninstalledVariant) : (sv as SchemaVariant);
  const key: UISchemaKey = {
    schemaId: variant.schemaId,
  };

  if ("schemaVariantId" in variant)
    key.schemaVariantId = variant.schemaVariantId;

  return key;
};

// Memoized fuzzy search instance to avoid recreating on every search
// Recreating the search instance on each debounced request was expensive
const fzfInstance = computed(() => {
  const filteredResults: UICategory[] = [];

  // Get the same filtering logic for consistency
  if (selectedFilter.value) {
    filteredResults.push(
      ...getCategoriesAndCountForFilterString(selectedFilter.value).categories,
    );
  } else {
    filteredResults.push(...categories.value);
  }

  const assets = filteredResults.flatMap((c) => c.assets);
  return useFzf(assets, (a: UIAsset) => `${a.name} ${a.uiCategory.name}`);
});

const filteredCategories = computed(() => {
  const filteredResults: UICategory[] = [];

  // Filtering by the selected top level category filter
  if (selectedFilter.value) {
    filteredResults.push(
      ...getCategoriesAndCountForFilterString(selectedFilter.value).categories,
    );
  } else {
    filteredResults.push(...categories.value);
  }

  if (debouncedSearchString.value !== "") {
    // Use the memoized fzf instance
    const results = fzfInstance.value.find(
      debouncedSearchString.value,
    ) as FzfResultItem<UIAsset>[];
    const items: UIAsset[] = results.map((fz) => fz.item);

    // reconstruct categories from the results (this is why asset.category exists)
    const categories: Record<string, UICategory> = {};
    items.forEach((item) => {
      let cat: UICategory | undefined = categories[item.uiCategory.name];
      if (!cat) {
        cat = {
          ...item.uiCategory,
          assets: [],
        };
        categories[item.uiCategory.name] = cat;
      }
      cat.assets.push(item);
    });
    filteredResults.splice(0, Infinity, ...Object.values(categories));
  }

  return filteredResults;
});

const filteredAssetsFlat = computed(() => {
  const assets: UIAsset[] = [];

  filteredCategories.value.forEach((category) => {
    category.assets.forEach((asset) => {
      assets.push(asset);
    });
  });

  return assets;
});

const showResults = computed(
  () => true, // !!(debouncedSearchString.value !== "" || selectedFilter.value),
);

const modalRef = ref<InstanceType<typeof Modal>>();
const searchRef = ref<InstanceType<typeof SiSearch>>();
const categoryTreeNodeRefs = ref<InstanceType<typeof TreeNode>[]>();

const fuzzySearchString = ref<string>("");
const debouncedSearchString = ref<string>("");
const selectedFilter = ref<string | undefined>(undefined);

// Debounce the search string updates to avoid expensive filtering on every keystroke
const updateDebouncedSearch = debounce(
  (value: string) => {
    debouncedSearchString.value = value;
  },
  500,
  { trailing: true, leading: false },
);

// Watch for changes to fuzzySearchString and selectedFilter
const justCleared = ref(false);

watch([fuzzySearchString, selectedFilter], ([newFuzzySearchString]) => {
  updateDebouncedSearch(newFuzzySearchString);
  fixCollapse();
});

watch(selectedAsset, (newSelectedAsset) => {
  if (newSelectedAsset === undefined || !categoryTreeNodeRefs.value) return;

  const selectedCategoryIndex = filteredCategories.value.findIndex((category) =>
    category.assets.find((asset) =>
      compareKeys(asset.key, newSelectedAsset.key),
    ),
  );

  const treeNode = categoryTreeNodeRefs.value[selectedCategoryIndex];
  treeNode?.toggleIsOpen(true);
});

const fixCollapse = () => {
  if (
    fuzzySearchString.value.length === 0 &&
    selectedFilter.value === undefined
  ) {
    categoryTreeNodeRefs.value?.forEach((node) => node.toggleIsOpen(false));
  } else {
    categoryTreeNodeRefs.value?.forEach((node) => node.toggleIsOpen(true));
  }
};

const toggleFilterTile = (name?: string) => {
  clearSelection();
  if (!name) selectedFilter.value = undefined;
  else if (selectedFilter.value === name) selectedFilter.value = undefined;
  else selectedFilter.value = name;
};

const isFilterSelected = (name: string) => {
  if (name === selectedFilter.value) return true;
  else if (name === "All" && selectedFilter.value === undefined) return true;
  else return false;
};

const open = () => {
  modalRef.value?.open();
  fuzzySearchString.value = "";
  debouncedSearchString.value = "";
  bannerClosed.value = false;
  toggleFilterTile();
  nextTick(() => {
    searchRef.value?.focusSearch();
    fixCollapse();
  });
};

const close = () => {
  modalRef.value?.close();
};

const pickIcon = (name: string): IconNames => {
  if (name.toLowerCase().includes("aws")) return "logo-aws";
  else if (name.toLowerCase().includes("coreos")) return "logo-coreos";
  else if (name.toLowerCase().includes("docker")) return "logo-docker";
  else if (name.toLowerCase().includes("fastly")) return "logo-fastly";
  // TODO(Wendy) - we need to fill out the rest of these icon lookups for the various categories/filters!
  else return "logo-si";
};

const foundCategoryMatch = (categoryName: string, category: UICategory) => {
  if (categoryName === "All") return true;
  if (categoryName === "Templates") return category.name === "Templates";
  return category.name.toLowerCase().includes(categoryName.toLowerCase());
};

const getCategoriesAndCountForFilterString = (categoryName = "All") => {
  let count = 0;
  let filteredCategories: UICategory[] = [];

  categories.value.forEach((category) => {
    const filtered: UICategory = { ...category, assets: [] };
    category.assets.forEach((asset) => {
      if (foundCategoryMatch(categoryName, category)) {
        count++;
        filtered.assets.push(asset);
      }
    });
    if (filtered.assets.length > 0) {
      filteredCategories.push(filtered);
    }
  });

  if (categoryName === "All") {
    filteredCategories = categories.value;
  }

  return {
    count,
    categories: filteredCategories,
  };
};

const componentFilters = computed((): AssetFilter[] => {
  const filters: AssetFilter[] = [
    {
      name: "All",
      icon: "logo-si", // TODO(Wendy) - different logo for this?
      count: getCategoriesAndCountForFilterString().count,
    },
    {
      name: "AWS",
      icon: pickIcon("aws"),
      count: getCategoriesAndCountForFilterString("aws").count,
      color: BRAND_COLOR_FILTER_HEX_CODES.AWS,
    },
    {
      name: "Fastly",
      icon: pickIcon("fastly"),
      count: getCategoriesAndCountForFilterString("fastly").count,
      color: BRAND_COLOR_FILTER_HEX_CODES.Fastly,
    },
    {
      name: "Templates",
      icon: "logo-si", // TODO(Wendy) - different logo for this?
      count: getCategoriesAndCountForFilterString("Templates").count,
    },
  ];
  return filters.filter((f) => f.count > 0);
});

const leftSideRef = ref<HTMLDivElement>();

const onClick = (e: MouseEvent | undefined) => {
  // Fixing the click exit handler
  if (!e) return;

  const target = e.target;

  if (
    !leftSideRef.value ||
    !(target instanceof Node) ||
    !document.contains(target)
  )
    return;

  if (
    (!showResults.value || !selectedAsset.value) &&
    !leftSideRef.value.contains(target) &&
    !justCleared.value
  ) {
    // clicking the empty area inside the modal
    close();
  }
};

defineExpose({ open, close, isOpen: modalRef.value?.isOpen });
</script>

<style lang="less" scoped>
div.grid.createcomponent {
  grid-template-columns: minmax(0, 70%) minmax(0, 30%);
  grid-template-rows: 1fr;
  grid-template-areas: "assets docs";
}
div.docs {
  grid-area: docs;
}
div.assets {
  grid-area: assets;
}
</style>
