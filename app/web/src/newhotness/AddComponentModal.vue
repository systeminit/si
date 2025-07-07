<template>
  <!-- NOTE: the Modal CSS for height in "max" doesn't work as we might expect -->
  <Modal ref="modalRef" noWrapper hideExitButton size="max" @click="onClick">
    <div class="h-[45svh]">
      <div
        :class="
          clsx(
            'grid createcomponent gap-xs [&>*]:border max-h-full',
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
            <!-- warning header for when user is on HEAD -->
            <div
              v-if="onHead && !bannerClosed"
              :class="
                clsx(
                  'flex flex-row items-center gap-xs p-2xs',
                  themeClasses('bg-action-100', 'bg-action-900'),
                )
              "
            >
              <Icon name="info-circle" />
              <div class="grow">
                Because you are currently on HEAD, when you create a component a
                new change set will be created.
              </div>
              <div
                :class="
                  clsx(
                    'underline cursor-pointer flex-none',
                    themeClasses(
                      'hover:text-action-500',
                      'hover:text-action-300',
                    ),
                  )
                "
              >
                Learn More
              </div>
              <IconButton
                iconTone="shade"
                icon="x"
                tooltip="Close"
                tooltipPlacement="top"
                @click="bannerClosed = true"
              />
            </div>
            <!-- Fuzzy search input, is focused when the Modal is opened-->
            <SiSearch
              ref="searchRef"
              v-model="fuzzySearchString"
              placeholder="Start typing to find components"
              :borderBottom="false"
              class="flex-none border"
              @blur="searchRef?.focusSearch()"
              @input="clearSelection"
              @keydown.enter.prevent="onEnter"
              @keydown.up.prevent="onUp"
              @keydown.down.prevent="onDown"
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
                :selected="!!(selectedFilter && selectedFilter === filter.name)"
                @click="toggleFilterTile(filter.name)"
              />
            </HorizontalScrollArea>
            <!-- Fuzzy search results list -->
            <div v-if="showResults" class="grow min-h-0 scrollable mb-xs">
              <TreeNode
                v-for="category in filteredCategories"
                :key="category.name"
                :class="themeClasses('bg-neutral-200', 'bg-neutral-700')"
                indentationSize="none"
                defaultOpen
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
            </div>
          </div>
        </div>
        <!-- Right side - documentation -->
        <template v-if="selectedAsset">
          <div class="docs scrollable border p-xs flex-1 overflow-auto">
            <h3>{{ selectedAsset.name }}</h3>
            <p>{{ selectedAsset.variant.link }}</p>
            <p><VueMarkdown :source="selectedAsset.variant.description" /></p>
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
  IconButton,
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
import { Fzf } from "fzf";
import { useRoute, useRouter } from "vue-router";
import { debounce } from "lodash-es";
import VueMarkdown from "vue-markdown-render";
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
import FilterTile from "./layout_components/FilterTile.vue";
import { assertIsDefined, Context, ExploreContext } from "./types";
import { componentTypes, routes, useApi } from "./api_composables";

const ctx: Context | undefined = inject("CONTEXT");
assertIsDefined(ctx);
const onHead = computed(() => ctx.onHead.value);
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
      el.scrollIntoView({ behavior: "smooth", block: "nearest" });
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
const onUp = () => {
  if (!showResults.value) return;

  if (selectionIndex.value === undefined) {
    selectionIndex.value = filteredAssetsFlat.value.length - 1;
  } else {
    selectionIndex.value--;
    if (selectionIndex.value < 0) {
      selectionIndex.value = filteredAssetsFlat.value.length - 1;
    }
  }
  selectAssetByIndex();
};
const onDown = () => {
  if (!showResults.value) return;

  if (selectionIndex.value === undefined) {
    selectionIndex.value = 0;
  } else {
    selectionIndex.value++;
    if (selectionIndex.value > filteredAssetsFlat.value.length - 1) {
      selectionIndex.value = 0;
    }
  }
  selectAssetByIndex();
};
const selectAssetByIndex = () => {
  if (
    selectionIndex.value !== undefined &&
    filteredAssetsFlat.value[selectionIndex.value]
  ) {
    selectAsset(filteredAssetsFlat.value[selectionIndex.value] as UIAsset);
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

  // i need the list of assets, not categories, to feed into the fuzzy search
  const assets = filteredResults.flatMap((c) => c.assets);

  if (debouncedSearchString.value !== "") {
    const fzf = new Fzf(assets, {
      casing: "case-insensitive",
      selector: (a: UIAsset) => `${a.name} ${a.uiCategory.name}`,
    });

    const results = fzf.find(debouncedSearchString.value);
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
  () => !!(debouncedSearchString.value !== "" || selectedFilter.value),
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

// Watch for changes to fuzzySearchString and update the debounced version
const justCleared = ref(false);

watch(fuzzySearchString, (newValue, oldValue) => {
  // Check if the search was cleared (likely by X button)
  if (oldValue && oldValue.length > 0 && newValue === "") {
    clearSelection();
    selectedFilter.value = undefined;
    justCleared.value = true;
    // Reset the flag after a short delay to prevent closing the modal
    setTimeout(() => {
      justCleared.value = false;
    }, 100);
  }

  updateDebouncedSearch(newValue);
});

const toggleFilterTile = (name: string) => {
  clearSelection();
  if (selectedFilter.value === name) selectedFilter.value = undefined;
  else selectedFilter.value = name;
};

const open = () => {
  modalRef.value?.open();
  searchRef.value?.focusSearch();
  fuzzySearchString.value = "";
  debouncedSearchString.value = "";
  bannerClosed.value = false;
  selectedAsset.value = undefined;
  selectedFilter.value = undefined;
  selectionIndex.value = undefined;
};

const close = () => {
  modalRef.value?.close();
};

const pickIcon = (name: string): IconNames => {
  if (name.toLowerCase().includes("aws")) return "logo-aws";
  else if (name.toLowerCase().includes("docker")) return "logo-docker";
  else if (name.toLowerCase().includes("coreos")) return "logo-coreos";
  // TODO(Wendy) - we need to fill out the rest of these icon lookups for the various categories/filters!
  else return "logo-si";
};

const getCategoriesAndCountForFilterString = (categoryName = "All") => {
  let count = 0;
  let filteredCategories: UICategory[] = [];

  categories.value.forEach((category) => {
    const filtered: UICategory = { ...category, assets: [] };
    category.assets.forEach((asset) => {
      // TODO(Wendy) - we probably need a better system than just string matching for some categories!
      if (
        categoryName === "All" ||
        category.name.toLowerCase().includes(categoryName.toLowerCase())
      ) {
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
      icon: "logo-si",
      count: getCategoriesAndCountForFilterString().count,
    },
  ];

  for (const [key, value] of Object.entries(BRAND_COLOR_FILTER_HEX_CODES)) {
    filters.push({
      name: key,
      icon: pickIcon(key),
      count: getCategoriesAndCountForFilterString(key).count,
      color: value,
    });
  }

  return filters;
});

const leftSideRef = ref<HTMLDivElement>();

const onClick = (e: MouseEvent | undefined) => {
  // Fixing the click exit handler
  if (!e) return;

  const target = e.target;

  if (!leftSideRef.value || !(target instanceof Node)) return;

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
