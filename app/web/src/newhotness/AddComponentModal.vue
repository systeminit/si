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
            <div
              v-if="showResults"
              ref="scrollRef"
              class="grow min-h-0 scrollable"
            >
              <div
                v-if="virtualItems.length === 0 && fuzzySearchString.length > 0"
                class="w-full h-full flex flex-row items-center justify-center"
              >
                <EmptyState
                  text="No components match your search."
                  icon="component"
                  noTopMargin
                />
              </div>
              <div
                class="w-full relative flex flex-col"
                :style="{
                  ['overflow-anchor']: 'none',
                  height: `${virtualListHeight}px`,
                }"
              >
                <AddComponentModalListRow
                  v-for="row in virtualItems"
                  :key="row.index"
                  :idx="row.index"
                  :style="{
                    height:
                      addComponentRowHeight(
                        categoryAndSchemaRows[row.index]?.type,
                      ) + 'px',
                    transform: `translateY(${row.start}px)`,
                  }"
                  :rowData="categoryAndSchemaRows[row.index]!"
                  :open="openFromIndex(row.index)"
                  :selected="
                    compareKeys(
                      selectedAsset?.key,
                      schemaFromVirtualRowIndex(row.index)?.key,
                    )
                  "
                  :submitted="
                    compareKeys(
                      selectedAsset?.key,
                      schemaFromVirtualRowIndex(row.index)?.key,
                    ) && api.inFlight.value
                  "
                  :createFailed="componentCreateFailed"
                  @click="() => assetClick(row.index)"
                />
              </div>
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
  IconNames,
  Modal,
  SiSearch,
  themeClasses,
  TruncateWithTooltip,
  TextPill,
} from "@si/vue-lib/design-system";
import { computed, inject, nextTick, ref, watch } from "vue";
import clsx from "clsx";
import { useQuery, useQueryClient } from "@tanstack/vue-query";
import { useRoute, useRouter } from "vue-router";
import { debounce } from "lodash-es";
import { FzfResultItem } from "fzf";
import { useVirtualizer } from "@tanstack/vue-virtual";
import {
  CategoryVariant,
  EntityKind,
  SchemaVariant,
  BifrostComponent,
  EddaComponent,
  AttributeTree,
  CachedDefaultVariant,
} from "@/workers/types/entity_kind_types";
import { getKind, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import AddComponentModalListRow, {
  AddComponentRowData,
} from "@/newhotness/AddComponentModalListRow.vue";
import { trackEvent } from "@/utils/tracking";
import { useFzf } from "./logic_composables/fzf";
import FilterTile from "./layout_components/FilterTile.vue";
import { assertIsDefined, Context, ExploreContext } from "./types";
import { componentTypes, routes, useApi } from "./api_composables";
import EmptyState from "./EmptyState.vue";
import MarkdownRender from "./MarkdownRender.vue";
import { pickBrandIconByString } from "./util";

const ctx: Context | undefined = inject("CONTEXT");
assertIsDefined(ctx);
const bannerClosed = ref(false);

const scrollRef = ref<HTMLDivElement | undefined>();

const selectedAsset = ref<UIAsset | undefined>(undefined);

const scrollToSelected = async () => {
  // First, wait one tick for the dom classes to update
  await nextTick();
  // Then, see if the element exists in the DOM
  const el = document.getElementsByClassName("add-component-selected-item")[0];
  if (el) {
    // If it does, scroll it to the center
    el.scrollIntoView({ block: "center" });
  } else {
    // Otherwise, we need to scroll using the virtualizer
    if (selectionIndex.value !== undefined && selectionIndex.value >= 0) {
      virtualList.value.scrollToIndex(
        controlIndexToVirtualizerIndex(selectionIndex.value),
        { align: "center" },
      );
    }
  }
};

const selectAsset = async (asset: UIAsset, noScroll?: boolean) => {
  selectedAsset.value = asset;
  // if you have a selected asset from this category open the category
  categoryIsOpen.value.add(asset.uiCategory.name);
  selectionIndex.value = filteredAssetsFlat.value.findIndex((a) =>
    compareKeys(a.key, asset.key),
  );

  if (!noScroll) {
    scrollToSelected();
  }
};
const clearSelection = () => {
  selectedAsset.value = undefined;
  selectionIndex.value = undefined;
};

const componentCreateFailed = ref(false);

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

  // Reset component create error
  componentCreateFailed.value = false;

  const key = selectedAsset.value.key;
  let params: componentTypes.ComponentIdType;
  if ("schemaVariantId" in key && key.schemaVariantId)
    params = {
      schemaType: "installed",
      schemaVariantId: key.schemaVariantId,
    };
  else
    params = {
      schemaType: "uninstalled",
      schemaId: key.schemaId,
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
  } else {
    // component create failed!
    componentCreateFailed.value = true;
    trackEvent("error-component-creation-failure", {
      payload,
      call,
    });
  }
};
const onUp = (e: KeyboardEvent) => {
  if (!showResults.value) return;

  // Reset component create error
  componentCreateFailed.value = false;

  const goByCategory =
    e.key !== "Tab" && (e.shiftKey || e.ctrlKey || e.metaKey);

  if (selectionIndex.value === undefined) {
    selectionIndex.value = filteredAssetsFlat.value.length - 1;
    selectAssetByIndex();
  } else {
    if (goByCategory) {
      selectFirstInNextCategory(selectionIndex.value, -1);
    } else {
      selectionIndex.value--;
      if (selectionIndex.value < 0) {
        selectionIndex.value = filteredAssetsFlat.value.length - 1;
      }
      selectAssetByIndex();
    }
  }
};
const onDown = (e: KeyboardEvent) => {
  if (!showResults.value) return;

  // Reset component create error
  componentCreateFailed.value = false;

  const goByCategory =
    e.key !== "Tab" && (e.shiftKey || e.ctrlKey || e.metaKey);

  if (selectionIndex.value === undefined) {
    selectionIndex.value = 0;
    selectAssetByIndex();
  } else {
    if (goByCategory) {
      selectFirstInNextCategory(selectionIndex.value, 1);
    } else {
      selectionIndex.value++;
      if (selectionIndex.value > filteredAssetsFlat.value.length - 1) {
        selectionIndex.value = 0;
      }
      selectAssetByIndex();
    }
  }
};
const changeFilterLeft = () => {
  let currentFilterIndex = componentFilters.value.findIndex(
    (filter) => filter.name === selectedFilter.value,
  );

  if (currentFilterIndex < 1) {
    currentFilterIndex = componentFilters.value.length - 1;
    const newFilter = componentFilters.value[currentFilterIndex];
    if (newFilter) {
      selectedFilter.value = newFilter.name;
    }
  } else {
    currentFilterIndex--;
    const newFilter = componentFilters.value[currentFilterIndex];
    if (newFilter) {
      selectedFilter.value = newFilter.name;
    }
  }
};
const changeFilterRight = () => {
  let currentFilterIndex = componentFilters.value.findIndex(
    (filter) => filter.name === selectedFilter.value,
  );

  if (currentFilterIndex === componentFilters.value.length - 1) {
    currentFilterIndex = 0;
    const newFilter = componentFilters.value[currentFilterIndex];
    if (newFilter) {
      selectedFilter.value = newFilter.name;
    }
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

  // Reset component create error
  componentCreateFailed.value = false;

  if (e.shiftKey) changeFilterLeft();
  else changeFilterRight();
};
const selectAssetByIndex = () => {
  if (selectionIndex.value !== undefined && selectionIndex.value >= 0) {
    const asset = filteredAssetsFlat.value[selectionIndex.value];
    if (asset) {
      selectAsset(asset);
    }
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
      selectAsset(nextCategory.assets[0], true);
    } else if (direction === 1 && firstCategoryAsset) {
      selectAsset(firstCategoryAsset, true);
    } else if (direction === -1 && lastCategoryAsset) {
      selectAsset(lastCategoryAsset, true);
    }
    scrollToSelected();
  }
};

type AssetFilter = {
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

export type UISchemaKey = {
  schemaId: string;
  schemaVariantId?: string;
};
const ffStore = useFeatureFlagsStore();
const defaultSchemaKey = makeKey(EntityKind.CachedDefaultVariant);
const defaultSchemas = useQuery({
  queryKey: defaultSchemaKey,
  queryFn: async () => {
    const schemas = await getKind<CachedDefaultVariant>(
      makeArgs(EntityKind.CachedDefaultVariant),
    );

    return schemas;
  },
});

const installedVariantsKey = makeKey(EntityKind.SchemaVariant);
const installedVariants = useQuery({
  queryKey: installedVariantsKey,
  queryFn: async () =>
    await getKind<SchemaVariant>(makeArgs(EntityKind.SchemaVariant)),
});

const categories = computed(() => {
  // do all the installed variants first, they always show
  const categories: Record<string, UICategory> = {};
  const installedSchemas: Set<string> = new Set();
  if (installedVariants.data.value) {
    installedVariants.data.value.forEach((variant) => {
      // Only show installed variants that are either the default or are editing.
      const members = ctx.schemaMembers?.value[variant.schemaId];
      if (
        members?.defaultVariantId !== variant.id &&
        members?.editingVariantId !== variant.id
      )
        return;

      const catName = variant.category || "SI";
      let category = categories[catName];
      if (!category) {
        category = {
          name: catName,
          color: variant.color,
          icon: pickBrandIconByString(variant.category),
          assets: [],
        };
      }
      installedSchemas.add(variant.schemaId);
      category.assets.push({
        variant,
        key: {
          schemaId: variant.schemaId,
          schemaVariantId: variant.schemaVariantId,
        },
        name: variant.displayName ?? "Unknown Name",
        uiCategory: category,
      });
      categories[catName] = category;
    });
  }

  // don't show a duplicated default schema if its already installed
  if (defaultSchemas.data.value) {
    defaultSchemas.data.value.forEach((variant) => {
      const catName = variant.category || "SI";
      let category = categories[catName];
      if (!category) {
        category = {
          name: catName,
          color: variant.color,
          icon: pickBrandIconByString(variant.category),
          assets: [],
        };
      }
      if (!installedSchemas.has(variant.id)) {
        category.assets.push({
          variant,
          key: {
            schemaId: variant.id,
          },
          name: variant.displayName,
          uiCategory: category,
        });
        categories[catName] = category;
      }
    });
  }

  return Object.values(categories).sort((a, b) => {
    const n1 = a.name.toUpperCase();
    const n2 = b.name.toUpperCase();
    if (n1 === n2) return 0;
    return n1 < n2 ? -1 : 1;
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

// Memoized fuzzy search instance to avoid recreating on every search
// Recreating the search instance on each debounced request was expensive
const fzfInstance = computed(() => {
  const filteredResults: UICategory[] = [];

  // Get the same filtering logic for consistency
  if (selectedFilter.value) {
    filteredResults.push(
      ...getCategoriesAndCountForFilterStrings(selectedFilter.value).categories,
    );
  } else {
    filteredResults.push(...categories.value);
  }

  const assets = filteredResults.flatMap((c) => c.assets);
  return useFzf(assets, (a: UIAsset) => `${a.name} ${a.uiCategory.name}`);
});

// PSA: reactive(new Set()) doesn't actually work!
const categoryIsOpen = ref<Set<string>>(new Set());
const filteredCategories = computed(() => {
  const filteredResults: UICategory[] = [];

  // Filtering by the selected top level category filter
  if (selectedFilter.value) {
    filteredResults.push(
      ...getCategoriesAndCountForFilterStrings(selectedFilter.value).categories,
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

const controlIndexToVirtualizerIndex = (idx: number) => {
  const asset = filteredAssetsFlat.value[idx];
  if (asset) {
    return categoryAndSchemaRows.value.findIndex(
      (row) => row.type === "schema" && compareKeys(asset.key, row.key),
    );
  } else {
    // default to the search bar if you don't find it!
    return -1;
  }
};

const showResults = computed(
  () => true, // !!(debouncedSearchString.value !== "" || selectedFilter.value),
);

const modalRef = ref<InstanceType<typeof Modal>>();
const searchRef = ref<InstanceType<typeof SiSearch>>();

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
  openAllCategories();
});

const closeAllCategories = () => {
  categoryIsOpen.value = new Set();
};

const openAllCategories = () => {
  filteredCategories.value.forEach((category) => {
    categoryIsOpen.value.add(category.name);
  });
};

const toggleFilterTile = (name?: string) => {
  clearSelection();
  if (!name || selectedFilter.value === name) {
    selectedFilter.value = undefined;
    nextTick(closeAllCategories);
  } else {
    selectedFilter.value = name;
    nextTick(openAllCategories);
  }
};

const isFilterSelected = (name: string) => {
  if (name === selectedFilter.value) return true;
  if (name === "All" && selectedFilter.value === undefined) return true;
  return false;
};

const resetModal = () => {
  fuzzySearchString.value = "";
  debouncedSearchString.value = "";
  bannerClosed.value = false;
  componentCreateFailed.value = false;
  closeAllCategories();
  selectedAsset.value = undefined;
  selectionIndex.value = undefined;
  toggleFilterTile();
  virtualList.value.scrollToIndex(0);
};

const open = () => {
  resetModal();
  modalRef.value?.open();
  nextTick(() => {
    searchRef.value?.focusSearch();
    virtualList.value.scrollToIndex(0);
  });
};

const close = () => {
  modalRef.value?.close();
};

const assetClick = (idx: number) => {
  // Reset component create error
  componentCreateFailed.value = false;

  const cat = categoryFromVirtualRowIndex(idx);
  if (cat) {
    if (categoryIsOpen.value.has(cat.name))
      categoryIsOpen.value.delete(cat.name);
    else categoryIsOpen.value.add(cat.name);
  }
  const schema = schemaFromVirtualRowIndex(idx);
  if (schema) {
    const asset = filteredAssetsFlat.value.find((a) =>
      compareKeys(a.key, schema.key),
    );
    if (asset) {
      if (compareKeys(asset.key, selectedAsset.value?.key)) onEnter();
      else selectAsset(asset, true);
    }
  }
};

const schemaFromVirtualRowIndex = (rowIdx: number) => {
  const maybeSchema = categoryAndSchemaRows.value[rowIdx];
  if (maybeSchema?.type === "schema") return maybeSchema;
  return undefined;
};

const categoryFromVirtualRowIndex = (rowIdx: number) => {
  const maybeCategory = categoryAndSchemaRows.value[rowIdx];
  if (maybeCategory?.type === "category") return maybeCategory;
  return undefined;
};

const openFromIndex = (idx: number) => {
  const cat = categoryFromVirtualRowIndex(idx);
  if (!cat) return false;
  return categoryIsOpen.value.has(cat.name);
};

const foundCategoryMatch = (categoryName: string, category: UICategory) => {
  if (categoryName === "All") return true;
  if (categoryName === "Templates") return category.name === "Templates";
  return category.name.toLowerCase().includes(categoryName.toLowerCase());
};

const getCategoriesAndCountForFilterStrings = (
  categoryFilterStrings: string | Array<string> = "All",
) => {
  const filterForOneString = (filterStr: string) => {
    const output: UICategory[] = [];
    categories.value.forEach((category) => {
      const filtered: UICategory = { ...category, assets: [] };
      category.assets.forEach((asset) => {
        if (foundCategoryMatch(filterStr, category)) {
          count++;
          filtered.assets.push(asset);
        }
      });
      if (filtered.assets.length > 0) {
        output.push(filtered);
      }
    });
    return output;
  };

  let count = 0;
  let filteredCategories: UICategory[] = [];

  if (categoryFilterStrings === "All") {
    filteredCategories = categories.value;
  } else if (Array.isArray(categoryFilterStrings)) {
    categoryFilterStrings.forEach((filterStr) => {
      filteredCategories.push(...filterForOneString(filterStr));
    });
  } else {
    filteredCategories = filterForOneString(categoryFilterStrings);
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
      count: getCategoriesAndCountForFilterStrings().count,
    },
    {
      name: "AWS",
      icon: pickBrandIconByString("aws"),
      count: getCategoriesAndCountForFilterStrings("aws").count,
      color: BRAND_COLOR_FILTER_HEX_CODES.AWS,
    },
    {
      name: "Microsoft",
      icon: "logo-azure",
      count: getCategoriesAndCountForFilterStrings("microsoft").count,
      color: BRAND_COLOR_FILTER_HEX_CODES.MS,
    },
    {
      name: "Hetzner",
      icon: pickBrandIconByString("hetzner"),
      count: getCategoriesAndCountForFilterStrings("hetzner").count,
      color: BRAND_COLOR_FILTER_HEX_CODES.Hetzner,
    },
    {
      name: "Fastly",
      icon: pickBrandIconByString("fastly"),
      count: getCategoriesAndCountForFilterStrings("fastly").count,
      color: BRAND_COLOR_FILTER_HEX_CODES.Fastly,
    },
    {
      name: "Templates",
      icon: "logo-si", // TODO(Wendy) - different logo for this?
      count: getCategoriesAndCountForFilterStrings("Templates").count,
    },
  ];

  if (ffStore.DIGITAL_OCEAN_ONBOARDING) {
    filters.splice(4, 0, {
      name: "DigitalOcean",
      icon: "logo-digital-ocean",
      count: getCategoriesAndCountForFilterStrings([
        "digitalocean",
        "digital ocean",
      ]).count,
      color: BRAND_COLOR_FILTER_HEX_CODES.DigitalOcean,
    });
  }

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

const categoryAndSchemaRows = computed(() => {
  const rows: AddComponentRowData[] = [];

  filteredCategories.value.forEach((category) => {
    rows.push({
      type: "category",
      name: category.name,
      icon: category.icon,
      color: category.color,
    });

    if (categoryIsOpen.value.has(category.name)) {
      category.assets.forEach((asset) => {
        rows.push({
          type: "schema",
          name: asset.name,
          color: category.color,
          key: asset.key,
          editing: !asset.variant.isLocked,
        });
      });
    }
  });

  return rows;
});

const CATEGORY_ROW_HEIGHT = 32;
const SCHEMA_ROW_HEIGHT = 28;

const addComponentRowHeight = (type?: string) => {
  if (type === "category") return CATEGORY_ROW_HEIGHT;
  else return SCHEMA_ROW_HEIGHT;
};

const virtualizerOptions = computed(() => ({
  count: categoryAndSchemaRows.value.length,
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  getScrollElement: () => scrollRef.value!,
  estimateSize: (i: number) =>
    addComponentRowHeight(categoryAndSchemaRows.value[i]?.type),
  getItemKey: (i: number) => {
    const row = categoryAndSchemaRows.value[i];
    if (row?.type === "category") {
      return `category-${i}`;
    }
    return `schema-${i}`;
  },
  overscan: 10,
}));

const virtualList = useVirtualizer(virtualizerOptions);

const virtualListHeight = computed(() => virtualList.value.getTotalSize());
const virtualItems = computed(() => virtualList.value.getVirtualItems());

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
