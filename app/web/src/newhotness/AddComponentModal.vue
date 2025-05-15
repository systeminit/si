<template>
  <!-- NOTE: the Modal CSS for height in "max" doesn't work as we might expect -->
  <Modal ref="modalRef" noWrapper hideExitButton size="max">
    <div
      :class="
        clsx(
          'grid createcomponent gap-xs [&>*]:border h-[45svh]',
          selectedAsset && 'grid grid-cols-2',
          themeClasses(
            '[&>*]:bg-shade-0 border-neutral-400 [&_*]:border-neutral-400',
            '[&>*]:bg-neutral-900 border-neutral-600 [&_*]:border-neutral-600',
          ),
        )
      "
    >
      <!-- Left side - search, filters, asset list -->
      <div class="assets flex flex-col gap-xs">
        <header class="text-md border-b p-xs flex-none">
          Create a component
        </header>

        <!-- I don't like that we have tos specify a height here it should
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
          <div class="grow min-h-0 scrollable">
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
                    selectedAsset?.id === asset.id &&
                      themeClasses(
                        'outline-action-500 bg-action-300',
                        'outline-action-300 bg-action-600',
                      ),
                  )
                "
                :color="asset.variant.color"
                @click="() => selectAsset(asset)"
              >
                <template #label>
                  <!-- TODO(Wendy) - style this text based on the fuzzy search! -->
                  {{ asset.name }}
                </template>
                <template v-if="selectedAsset?.id === asset.id" #icons>
                  <TextPill tighter class="text-xs">Enter to add</TextPill>
                </template>
              </TreeNode>
            </TreeNode>
          </div>
        </div>
      </div>
      <!-- Right side - documentation and attributes -->
      <template v-if="selectedAsset">
        <div class="docs scrollable border p-xs">
          <h3>{{ selectedAsset.name }}</h3>
          <p>{{ selectedAsset.variant.link }}</p>
          <p>{{ selectedAsset.variant.description }}</p>
        </div>
        <div class="props scrollable border p-xs">
          <template v-if="'props' in selectedAsset.variant">
            <ol>
              <li v-for="prop in selectedAssetProps" :key="prop.id">
                {{ prop.name }}
              </li>
            </ol>
          </template>
        </div>
      </template>
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
} from "@si/vue-lib/design-system";
import { computed, inject, ref } from "vue";
import clsx from "clsx";
import { useQuery } from "@tanstack/vue-query";
import { Fzf } from "fzf";
import { useRoute, useRouter } from "vue-router";
import TextPill from "@/components/TextPill.vue";
import {
  BifrostSchemaVariantCategories,
  PropOnVariant,
} from "@/workers/types/dbinterface";
import { bifrost, makeArgs, makeKey } from "@/store/realtime/heimdall";
import { CategoryVariant } from "@/store/components.store";
import FilterTile from "./layout_components/FilterTile.vue";
import { assertIsDefined, Context } from "./types";
import { keyEmitter } from "./logic_composables/emitters";
import {
  ComponentIdType,
  CreateComponentPayload,
  createComponentPayload,
  routes,
  useApi,
} from "./api_composables";

const ctx: Context | undefined = inject("CONTEXT");
assertIsDefined(ctx);
const onHead = computed(() => ctx.onHead.value);
const bannerClosed = ref(false);

const selectedAsset = ref<UIAsset | undefined>(undefined);
const selectAsset = (asset: UIAsset) => {
  if (selectedAsset.value?.id === asset.id) selectedAsset.value = undefined;
  else selectedAsset.value = asset;
};

const selectedAssetProps = computed<PropOnVariant[]>(() => {
  if (!selectedAsset.value) return [] as PropOnVariant[];
  if (!("props" in selectedAsset.value.variant)) return [] as PropOnVariant[];
  return selectedAsset.value.variant.props
    .filter((p) => !p.hidden)
    .filter(
      (p) => p.path.startsWith("/root/domain") && p.path !== "/root/domain",
    );
});

const props = defineProps<{
  viewId: string;
}>();

const viewId = computed(() => props.viewId);

const route = useRoute();
const router = useRouter();
const api = useApi();
keyEmitter.on("Enter", async () => {
  if (!selectedAsset.value) return;
  const schemaType = selectedAsset.value.type;
  let params: ComponentIdType;
  if (schemaType === "installed")
    params = {
      schemaType,
      schemaVariantId: selectedAsset.value.variant.schemaVariantId,
    };
  else
    params = {
      schemaType,
      schemaVariantId: selectedAsset.value.variant.schemaId,
    };

  // TODO "force changeset"
  const payload = createComponentPayload(params);
  const call = api.endpoint<{ componentId: string }>(routes.CreateComponent, {
    viewId: viewId.value,
  });
  const resp = await call.post<CreateComponentPayload>(payload);
  if (api.ok(resp)) {
    const params = {
      ...route.params,
      componentId: resp.data.componentId,
    };
    router.push({
      name: "new-hotness-component",
      params,
      // TODO querystring for "was i on head?"
    });
  }
});

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

type UIAsset = CategoryVariant & {
  name: string;
  category: UICategoryInfo;
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
    const categoryInfo: UICategoryInfo = {
      name: rawCategory.displayName,
      color: firstSV.variant.color,
      icon: pickIcon(rawCategory.displayName),
    };
    const category: UICategory = {
      ...categoryInfo,
      assets: [],
    };
    rawCategory.schemaVariants.forEach((sv) => {
      const asset = {
        ...sv,
        name: sv.variant.displayName ?? sv.variant.schemaName,
        category: categoryInfo,
      };
      category.assets.push(asset);
    });
    return category;
  });
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

  // i need the list of assets, not categories, to feed into the fuzzy search
  const assets = filteredResults.flatMap((c) => c.assets);

  if (fuzzySearchString.value !== "") {
    const fzf = new Fzf(assets, {
      casing: "case-insensitive",
      selector: (a: UIAsset) => `${a.name} ${a.category.name}`,
    });

    const results = fzf.find(fuzzySearchString.value);
    const items: UIAsset[] = results.map((fz) => fz.item);

    // reconstruct categories from the results (this is why asset.category exists)
    const categories: Record<string, UICategory> = {};
    items.forEach((item) => {
      let cat: UICategory | undefined = categories[item.category.name];
      if (!cat) {
        cat = {
          ...item.category,
          assets: [],
        };
        categories[item.category.name] = cat;
      }
      cat.assets.push(item);
    });
    filteredResults.splice(0, Infinity, ...Object.values(categories));
  }

  return filteredResults;
});

const modalRef = ref<InstanceType<typeof Modal>>();
const searchRef = ref<InstanceType<typeof SiSearch>>();

const fuzzySearchString = ref<string>("");
const selectedFilter = ref<string | undefined>(undefined);

const toggleFilterTile = (name: string) => {
  if (selectedFilter.value === name) selectedFilter.value = undefined;
  else selectedFilter.value = name;
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

defineExpose({ open, close });
</script>

<style lang="less" scoped>
div.grid.createcomponent {
  grid-template-columns: minmax(0, 70%) minmax(0, 30%);
  grid-template-rows: 1fr 1fr;
  grid-template-areas:
    "assets docs"
    "assets props";
}
div.docs {
  grid-area: docs;
}
div.props {
  grid-area: props;
}
div.assets {
  grid-area: assets;
}
</style>
