<template>
  <Modal ref="modalRef" noWrapper hideExitButton size="max">
    <div
      :class="
        clsx(
          'createcomponent gap-xs [&>*]:border',
          selectedAsset && 'grid grid-cols-2',
          themeClasses(
            '[&>*]:bg-shade-0 border-neutral-400 [&_*]:border-neutral-400',
            '[&>*]:bg-neutral-900 border-neutral-600 [&_*]:border-neutral-600',
          ),
        )
      "
    >
      <!-- Left side - search, filters, asset list -->
      <div class="flex flex-col gap-xs [&>*]:w-full">
        <header class="text-md border-b p-xs flex-none">
          Create a component
        </header>

        <div class="flex flex-col gap-xs grow p-xs max-h-[60vh]">
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
              Because you are currently on Head, when you create a component a
              new change set will be created.
            </div>
            <div
              :class="
                clsx(
                  'underline cursor-pointer',
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
          >
            <template #right>
              <div
                :class="
                  clsx(
                    'flex flex-row flex-none gap-3xs items-center text-2xs pr-2xs',
                    themeClasses('text-shade-100', 'text-shade-0'),
                  )
                "
              >
                <TextPill tighter>Up</TextPill>
                <TextPill tighter>Down</TextPill>
                <div class="leading-snug">to navigate</div>
              </div>
            </template>
          </SiSearch>
          <!-- Fuzzy search category filters -->
          <HorizontalScrollArea
            hideScrollbar
            class="flex flex-row gap-xs flex-none"
          >
            <ComponentFilterTile
              v-for="filter in componentFilters"
              ref="componentFilterTilesRef"
              :key="filter.name"
              :filter="filter"
              :selected="selectedFilter && selectedFilter.name === filter.name"
              @click="toggleFilterTile(filter.name)"
            />
          </HorizontalScrollArea>
          <!-- Fuzzy search results list -->
          <div
            v-if="fuzzySearchString !== '' || selectedFilter"
            class="grow min-h-0 scrollable"
          >
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
                :key="asset.id"
                :class="
                  clsx(
                    'hover:outline hover:z-10 hover:-outline-offset-1 hover:outline-1',
                    themeClasses(
                      'bg-shade-0 hover:outline-action-500',
                      'bg-neutral-800 hover:outline-action-300',
                    ),
                  )
                "
                :color="asset.color"
              >
                <template #label>
                  <!-- TODO(Wendy) - style this text based on the fuzzy search! -->
                  {{ asset.name }}
                </template>
              </TreeNode>
            </TreeNode>
          </div>
        </div>
      </div>
      <!-- Right side - documentation and attributes -->
      <div
        v-if="selectedAsset"
        :class="
          clsx(
            'information flex flex-col gap-xs [&>*]:border [&>*]:p-xs [&>*]:grow',
            themeClasses(
              '[&>*]:border-neutral-400',
              '[&>*]:border-neutral-600',
            ),
          )
        "
      >
        <div>Documentation goes here</div>
        <div>Attributes go here</div>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
// import { Fzf } from "fzf"; // TODO(Wendy) - import this and use it!
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
} from "@si/vue-lib/design-system";
import { computed, inject, ref } from "vue";
import clsx from "clsx";
import { useQuery } from "@tanstack/vue-query";
import TextPill from "@/components/TextPill.vue";
import { BifrostSchemaVariantCategories } from "@/workers/types/dbinterface";
import { bifrost, makeArgs, makeKey } from "@/store/realtime/heimdall";
import ComponentFilterTile from "./ComponentFilterTile.vue";
import { Context } from "./types";

const ctx: Context | undefined = inject("CONTEXT");
const onHead = computed(() => ctx?.onHead ?? false); // TODO(Wendy) - doesn't seem to be working? Not sure why.
const bannerClosed = ref(false);
const selectedAsset = ref<UIAsset | undefined>(undefined);

export type AssetFilter = {
  name: string;
  icon: IconNames;
  count: number;
  color?: string;
};

type UICategory = {
  name: string;
  icon?: IconNames;
  color: string;
  assets: UIAsset[];
};

type UIAsset = {
  id: string;
  name: string;
  color: string;
};

const queryKey = makeKey("SchemaVariantCategories");
const schemaVariantCategoriesOverBifrost =
  useQuery<BifrostSchemaVariantCategories | null>({
    queryKey,
    queryFn: async () =>
      await bifrost<BifrostSchemaVariantCategories>(
        makeArgs("SchemaVariantCategories"),
      ),
  });

const categories = computed(() => {
  const rawCategoryData =
    schemaVariantCategoriesOverBifrost.data.value?.categories ?? [];

  return rawCategoryData.map((rawCategory) => {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const firstSV = rawCategory.schemaVariants[0]!;
    const category: UICategory = {
      name: rawCategory.displayName,
      color: firstSV.variant.color,
      icon: pickIcon(rawCategory.displayName),
      assets: [],
    };
    rawCategory.schemaVariants.forEach((sv) => {
      const asset = {
        id: sv.id,
        name: sv.variant.displayName ?? sv.variant.schemaName,
        color: sv.variant.color,
      };
      category.assets.push(asset);
    });
    return category;
  });
});

const filteredCategories = computed(() => {
  const filteredResults = [];

  // Filtering by the selected top level category filter
  if (selectedFilter.value) {
    filteredResults.push(
      ...getCategoriesAndCountForFilterString(selectedFilter.value.name)
        .categories,
    );
  } else {
    filteredResults.push(...categories.value);
  }

  // Filtering by fuzzy search string
  if (fuzzySearchString.value !== "") {
    // TODO(Wendy) - filter things down by fuzzy search!
  }

  return filteredResults;
});

const modalRef = ref<InstanceType<typeof Modal>>();
const searchRef = ref<InstanceType<typeof SiSearch>>();
const componentFilterTilesRef =
  ref<InstanceType<typeof ComponentFilterTile>[]>();

const fuzzySearchString = ref<string>("");
const selectedFilter = ref<AssetFilter | undefined>(undefined);

const toggleFilterTile = (name: string) => {
  const selectedTile = componentFilterTilesRef.value?.find(
    (filter) => filter.$props.filter.name === name,
  )?.$props.filter;
  if (!selectedTile) return;
  else if (selectedFilter.value?.name === selectedTile.name) {
    selectedFilter.value = undefined;
  } else {
    selectedFilter.value = selectedTile;
  }
};

const open = () => {
  modalRef.value?.open();
  searchRef.value?.focusSearch();
  fuzzySearchString.value = "";
  bannerClosed.value = false;
  selectedAsset.value = undefined;
  selectedFilter.value = undefined;
};

const close = () => {
  modalRef.value?.close();
};

const pickIcon = (name: string): IconNames => {
  if (name.toLowerCase().includes("aws")) return "logo-aws";
  else if (name.toLowerCase().includes("docker")) return "logo-docker";
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

defineExpose({ open, close });
</script>

<style lang="css" scoped>
div.grid.createcomponent {
  grid-template-columns: minmax(0, 70%) minmax(0, 30%);
  grid-template-rows: 100%;
}
</style>
