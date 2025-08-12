<template>
  <div class="h-full flex flex-col">
    <div
      :class="
        clsx(
          'flex-none',
          'bulk-header flex flex-row items-center gap-xs',
          'mx-[-12px]', // pull this banner beyond the margins of its container's style [&>div]:mx-[12px]
          themeClasses('bg-white', 'bg-neutral-800'),
        )
      "
    >
      <IconButton
        tooltip="Close (Esc)"
        tooltipPlacement="top"
        class="border-0 mr-2em"
        icon="x"
        size="sm"
        iconIdleTone="shade"
        iconTone="shade"
        @click="() => emit('close')"
      />
      <div class="flex-none text-sm">Edit selected components</div>
    </div>
    <div :class="clsx('grow min-h-0 flex flex-row gap-xs mt-xs items-stretch')">
      <ul
        :class="
          clsx(
            'my-2xs', // matching <AttributeChildLayout>
            'scrollable min-h-0',
            'w-1/4 flex-none flex flex-col gap-xs border p-sm',
            themeClasses('border-neutral-300', 'border-neutral-600'),
            themeClasses(
              'bg-shade-0 border-neutral-400',
              'bg-neutral-800 border-neutral-600',
            ),
          )
        "
      >
        <li class="text-sm mb-2xs text-neutral-400">
          Components selected for editing
        </li>
        <!-- i took these styles and html nesting from connection panel, we should create a component that does this -->
        <li
          v-for="[idx, component] in Object.entries(
            exploreContext.selectedComponentsMap.value,
          )"
          :key="component.id"
          :class="clsx('ml-xs', 'flex flex-col gap-2xs')"
        >
          <div
            :class="
              clsx(
                'flex flex-row items-center gap-xs [&>*]:text-sm [&>*]:font-bold',
                themeClasses(
                  '[&>*]:border-neutral-400',
                  '[&>*]:border-neutral-600',
                ),
              )
            "
          >
            <input type="checkbox" checked @click="() => deselect(idx)" />
            <MinimizedComponentQualificationStatus
              :component="component"
              noText
            />
            <TextPill
              mono
              :class="
                clsx(
                  'min-w-0',
                  themeClasses('text-green-light-mode', 'text-green-dark-mode'),
                )
              "
            >
              <TruncateWithTooltip>
                {{ component.schemaVariantName }}
              </TruncateWithTooltip>
            </TextPill>
            <TextPill mono class="text-purple min-w-0">
              <TruncateWithTooltip>
                {{ component.name }}
              </TruncateWithTooltip>
            </TextPill>
          </div>
          <div v-if="errors[component.id]" class="mb-sm">
            <div
              v-for="[path, err] in Object.entries(errors[component.id]!)"
              :key="path"
              :class="
                clsx(
                  'border p-xs',
                  themeClasses(
                    'border-destructive-900 bg-destructive-200 text-black',
                    'border-destructive-300 bg-newhotness-destructive text-white',
                  ),
                )
              "
            >
              <p class="text-sm">{{ path }}</p>
              <p
                :class="
                  clsx(
                    'text-xs',
                    themeClasses(
                      'text-destructive-900',
                      'text-destructive-300',
                    ),
                  )
                "
              >
                {{ err }}
              </p>
            </div>
          </div>
        </li>
      </ul>
      <DelayedLoader v-if="treesPending" :size="'full'" />
      <div
        v-else-if="commonTree.domain || commonTree.secrets"
        class="grow min-h-0 my-2xs"
      >
        <!-- styles taken from CollapsingFlexItem -->
        <div
          :class="
            clsx(
              'h-full scrollable',
              'flex flex-col items-stretch',
              'border overflow-hidden mb-[-1px]', // basis-0 makes items take equal size when multiple are open
              '[&>dl]:m-xs', // pad the child attributes
              themeClasses('border-neutral-300', 'border-neutral-600'),
              themeClasses(
                'bg-shade-0 border-neutral-400',
                'bg-neutral-800 border-neutral-600',
              ),
            )
          "
        >
          <h3
            :class="
              clsx(
                'group/header',
                'text-sm flex-none h-lg flex items-center px-xs m-0',
                'border-b',
                themeClasses('border-neutral-300', 'border-neutral-600'),
              )
            "
          >
            Shared attributes
          </h3>
          <AttributeChildLayout v-if="commonTree.domain">
            <template #header><span class="text-sm">domain</span></template>
            <ComponentAttribute
              v-for="child in commonTree.domain.children"
              :key="child.id"
              :component="componentMap[child.componentId]!"
              :attributeTree="child"
              @save="save"
              @add="add"
              @set-key="setKey"
              @remove-subscription="removeSubscription"
              @delete="remove"
            />
          </AttributeChildLayout>
          <AttributeChildLayout
            v-if="commonTree.secrets && commonTree.secrets.children.length > 0"
          >
            <template #header><span class="text-sm">secrets</span></template>
            <ComponentSecretAttribute
              v-for="secret in commonTree.secrets.children"
              :key="secret.id"
              :component="componentMap[secret.componentId]!"
              :attributeTree="secret"
            />
          </AttributeChildLayout>
          <p v-else class="italic text-center mt-md">
            The selected components do not share any common attributes.
          </p>
        </div>
      </div>
      <div
        :class="
          clsx(
            'my-2xs', // matching <AttributeChildLayout>
            'scrollable min-h-0',
            'w-1/5 flex-none flex flex-col gap-xs scrollable border',
            themeClasses('border-neutral-300', 'border-neutral-600'),
            themeClasses(
              'bg-shade-0 border-neutral-400',
              'bg-neutral-800 border-neutral-600',
            ),
          )
        "
      >
        <h3
          :class="
            clsx(
              'group/header',
              'text-sm flex-none flex items-center h-lg px-xs m-0',
              'border-b',
              themeClasses('border-neutral-300', 'border-neutral-600'),
            )
          "
        >
          <template v-if="selectedPathName">
            {{ selectedPathName }} values
          </template>
        </h3>
        <div class="m-sm flex flex-col gap-xs">
          <template v-if="historyValueData">
            <template
              v-for="[valueKey, desc] in Object.entries(historyValueData)"
              :key="valueKey"
            >
              <AttributeValueBox>
                <template #components>
                  <AttrComponentList :components="desc.components" />
                </template>
                <AttrValue
                  strikeout
                  :isSecret="desc.isSecret"
                  :path="valueKey.split('|')[1]?.replace(/^\//, '')"
                  :value="valueKey.split('|')[2]"
                  :componentName="valueKey.split('|')[0]"
                />
              </AttributeValueBox>
            </template>
          </template>
          <template v-if="pathValueData">
            <template
              v-for="[valueKey, desc] in Object.entries(pathValueData)"
              :key="valueKey"
            >
              <AttributeValueBox>
                <template #components>
                  <AttrComponentList :components="desc.components" />
                </template>
                <AttrValue
                  :isSecret="desc.isSecret"
                  :path="valueKey.split('|')[1]?.replace(/^\//, '')"
                  :value="valueKey.split('|')[2]"
                  :componentName="valueKey.split('|')[0]"
                />
              </AttributeValueBox>
            </template>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { useQueries } from "@tanstack/vue-query";
import {
  computed,
  inject,
  onBeforeUnmount,
  onMounted,
  provide,
  reactive,
  ref,
  watch,
} from "vue";
import {
  themeClasses,
  IconButton,
  TruncateWithTooltip,
  TextPill,
} from "@si/vue-lib/design-system";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import {
  AttributeTree,
  ComponentInList,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import {
  attributeEmitter,
  keyEmitter,
} from "@/newhotness/logic_composables/emitters";
import { nonNullable } from "@/utils/typescriptLinter";
import ComponentAttribute, {
  NewChildValue,
} from "@/newhotness/layout_components/ComponentAttribute.vue";
import ComponentSecretAttribute from "@/newhotness/layout_components/ComponentSecretAttribute.vue";
import AttrValue from "@/newhotness/layout_components/AttrValue.vue";
import AttributeValueBox from "@/newhotness/layout_components/AttributeValueBox.vue";
import AttributeChildLayout from "@/newhotness/layout_components/AttributeChildLayout.vue";
import DelayedLoader from "@/newhotness/layout_components/DelayedLoader.vue";
import { AttributePath, ComponentId } from "@/api/sdf/dal/component";
import { PropKind } from "@/api/sdf/dal/prop";
import {
  arrayAttrTreeIntoTree,
  AttrTree,
  makeAvTree,
  makeSavePayload,
} from "../logic_composables/attribute_tree";
import {
  componentTypes,
  DoResponse,
  ok,
  routes,
  UseApi,
  useApi,
} from "../api_composables";
import { useContext } from "../logic_composables/context";
import MinimizedComponentQualificationStatus from "../MinimizedComponentQualificationStatus.vue";
import AttrComponentList from "../layout_components/AttrComponentList.vue";
import {
  assertIsDefined,
  AttributeInputContext,
  ExploreContext,
} from "../types";

const ctx = useContext();
const exploreContext = inject<ExploreContext>("EXPLORE_CONTEXT");
assertIsDefined<ExploreContext>(exploreContext);

const deselect = (index: number) => {
  emit("deselect", index);
};

const componentMap = computed(() =>
  Object.values(exploreContext.selectedComponentsMap.value).reduce(
    (obj, component) => {
      obj[component.id] = component;
      return obj;
    },
    {} as Record<string, ComponentInList>,
  ),
);
const componentIds = computed(() => Object.keys(componentMap.value));

const makeKey = useMakeKey();
const makeArgs = useMakeArgs();

// first get all of the relevant AV trees
const queries = computed(() =>
  componentIds.value.map((id) => {
    return {
      queryKey: makeKey(EntityKind.AttributeTree, id),
      queryFn: async () =>
        await bifrost<AttributeTree>(makeArgs(EntityKind.AttributeTree, id)),
    };
  }),
);
const avTrees = useQueries({
  queries,
});

const treesPending = computed(() =>
  avTrees.value.some((t) => t.status === "pending"),
);

// now turn them all into the AttrTree type, starting at domain
type Trees = Array<{
  root: AttrTree | undefined;
  domain: AttrTree | undefined;
  secrets: AttrTree | undefined;
  componentName: string;
  schemaName: string;
}>;
const trees = computed<Trees>(() => {
  return avTrees.value
    .map((t) => t.data)
    .filter(nonNullable)
    .map((t) => {
      const rootId = Object.keys(t.treeInfo).find((avId) => {
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        const av = t.treeInfo[avId]!;
        if (!av.parent) return true;
        return false;
      });
      if (!rootId) return null;

      const tree = makeAvTree(t, rootId, false);
      const domain = tree.children.find((c) => c.prop?.name === "domain");
      const secrets = tree.children.find((c) => c.prop?.name === "secrets");
      return {
        domain,
        secrets,
        root: tree,
        componentName: t.componentName,
        schemaName: t.schemaName,
      };
    })
    .filter(nonNullable);
});

const errors = reactive<Record<ComponentId, Record<AttributePath, string>>>({});
const upsertError = (id: ComponentId, path: AttributePath, error?: string) => {
  let paths = errors[id];
  if (!paths) {
    paths = {} as Record<AttributePath, string>;
    errors[id] = paths;
  }
  if (error) paths[path] = error;
  else delete paths[path];
};

const history = reactive<Record<AttributePath, PathValueData>>(
  {} as Record<AttributePath, PathValueData>,
);
const showHistory = reactive<Record<AttributePath, boolean>>(
  {} as Record<AttributePath, boolean>,
);

provide<AttributeInputContext>("ATTRIBUTEINPUT", { blankInput: true });

const setHistory = (path: AttributePath) => {
  showHistory[path] = true;
  historyValueData.value = history[path];
};

// record history once we have the intitial set of trees.
const onlyOnceStop = watch(
  trees,
  () => {
    if (trees.value.length === 0) return;

    const names = trees.value.reduce((obj, t) => {
      if (!t.root) return obj;
      obj[t.root.componentId] = {
        componentName: t.componentName,
        schemaName: t.schemaName,
      };
      return obj;
    }, {} as Record<string, { componentName: string; schemaName: string }>);

    const children = trees.value.flatMap((t) => {
      const d = t.domain || { children: [] as AttrTree[] };
      const s = t.secrets || { children: [] as AttrTree[] };
      return [...d.children].concat([...s.children]);
    });

    while (children.length > 0) {
      const child = children.shift();
      if (!child) continue;

      const av = child.attributeValue;
      const source = (av.externalSources || [])[0];
      const v =
        typeof av.value === "object" ? JSON.stringify(av.value) : av.value;
      const valKey: ValueKey = `${source?.componentName}|${source?.path}|${v}`;
      let vals = history[av.path];
      if (!vals) {
        vals = {} as PathValueData;
        history[av.path] = vals;
      }
      let data = vals[valKey];
      if (!data) {
        data = {
          isSecret: !!av.secret,
          components: [],
        } as ComponentsWithValue;
        vals[valKey] = data;
      }
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      data.components.push(names[child.componentId]!);

      children.push(...child.children);
    }

    if (onlyOnceStop) {
      onlyOnceStop();
    }
  },
  { immediate: true },
);

// now filter them to the common AV paths present in all trees
// the *actual* AVs will be from the very last component that has them
// because of how we are using `pathMap`
const commonTree = computed<{
  domain: AttrTree | undefined;
  secrets: AttrTree | undefined;
}>(() => {
  const pathMap: Record<string, AttrTree> = {};
  const commonPaths = new Set<string>();

  trees.value.forEach((componentAttrs) => {
    const walking = [componentAttrs.domain, componentAttrs.secrets];
    const paths = new Set<string>();
    while (walking.length > 0) {
      const attr = walking.shift();
      if (!attr) continue;
      pathMap[attr.attributeValue.path] = attr;
      paths.add(attr.attributeValue.path);
      walking.push(...attr.children);
    }
    if (commonPaths.size === 0) {
      paths.forEach((p) => commonPaths.add(p));
    } else {
      const common = new Set<string>();
      commonPaths.forEach((p) => {
        if (paths.has(p)) {
          common.add(p);
        }
      });
      commonPaths.clear();
      common.forEach((p) => commonPaths.add(p));
    }
  });

  // now remove any of the paths from the first component, that isn't in the rest
  const matches = [...commonPaths]
    .map((path) => pathMap[path])
    .filter(nonNullable)
    .map((t) => {
      return { ...t };
    });
  const map = matches.reduce((obj, attr) => {
    obj[attr.id] = attr;
    return obj;
  }, {} as Record<string, AttrTree>);

  const matchesAsTree = arrayAttrTreeIntoTree(matches, map);

  const domain = Object.values(matchesAsTree).find(
    (t) => t.prop?.name === "domain",
  );
  const secrets = Object.values(matchesAsTree).find(
    (t) => t.prop?.name === "secrets",
  );
  return {
    domain: matchesAsTree[domain?.id || ""],
    secrets: matchesAsTree[secrets?.id || ""],
  };
});

type ApiVal = {
  value: string;
  propKind: PropKind;
  connectingComponentId?: ComponentId;
};

const saving = ref(0);

type ApiArgs = { id: ComponentId };
type SuccessResp = { success: boolean };
const createCalls = () =>
  componentIds.value.map((componentId) => {
    const saveApi = useApi<ApiArgs>(ctx);
    const call = saveApi.endpoint<SuccessResp>(
      routes.UpdateComponentAttributes,
      { id: componentId },
    );
    return call;
  });

const createPayload = (path: AttributePath, vals: ApiVal) =>
  makeSavePayload(path, vals.value, vals.propKind, vals.connectingComponentId);

const handleErrors = (
  path: AttributePath,
  resps: Array<DoResponse<SuccessResp, ApiArgs>>,
): void => {
  resps.forEach((resp) => {
    let err;
    if (!ok(resp.req)) {
      err = resp.errorMessage;
    }
    upsertError(resp.endpointArgs.id, path, err);
  });
};

const add = async (
  _: UseApi,
  attributeTree: AttrTree,
  value: NewChildValue,
) => {
  if (ctx.onHead.value) throw new Error("Must be on a change set");

  const apis = createCalls();

  const appendPath = `${attributeTree.attributeValue.path}/-` as AttributePath;
  const calls = apis.map(async (call) => {
    const payload = {
      [appendPath]: value,
    };
    return call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
  });

  saving.value = calls.length;
  const resps = await Promise.all(calls);
  saving.value = 0;
  handleErrors(appendPath, resps);
};

const setKey = async (
  attributeTree: AttrTree,
  key: string,
  value: NewChildValue,
) => {
  if (ctx.onHead.value) throw new Error("Must be on a change set");

  const apis = createCalls();

  const appendPath =
    `${attributeTree.attributeValue.path}/${key}` as AttributePath;
  const calls = apis.map(async (call) => {
    const payload = {
      [appendPath]: value,
    };
    return call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
  });

  saving.value = calls.length;
  const resps = await Promise.all(calls);
  setHistory(attributeTree.attributeValue.path);
  saving.value = 0;
  handleErrors(appendPath, resps);
};

const save = async (
  path: AttributePath,
  value: string,
  propKind: PropKind,
  connectingComponentId?: ComponentId,
) => {
  // TODO force change set if on HEAD when starting
  if (ctx.onHead.value) throw new Error("Must be on a change set");

  // one API call per component
  const apis = createCalls();

  const calls = apis.map(async (call) => {
    const payload = createPayload(path, {
      value,
      propKind,
      connectingComponentId,
    });

    return await call.put<componentTypes.UpdateComponentAttributesArgs>(
      payload,
    );
  });
  saving.value = calls.length;
  const resps = await Promise.all(calls);
  setHistory(path);
  saving.value = 0;
  handleErrors(path, resps);
};

const removeSubscription = async (path: AttributePath) => {
  if (ctx.onHead.value) throw new Error("Must be on a change set");

  const apis = createCalls();

  const calls = apis.map(async (call) => {
    const payload = {
      [path]: {
        $source: null,
      },
    };
    return call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
  });

  saving.value = calls.length;
  const resps = await Promise.all(calls);
  setHistory(path);
  saving.value = 0;
  handleErrors(path, resps);
};

const remove = async (path: AttributePath) => {
  const apis = createCalls();
  const payload: componentTypes.UpdateComponentAttributesArgs = {};
  const calls = apis.map(async (call) => {
    payload[path] = { $source: null };
    return call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
  });

  saving.value = calls.length;
  const resps = await Promise.all(calls);
  setHistory(path);
  saving.value = 0;
  handleErrors(path, resps);
};

const onEscape = () => {
  emit("close");
};
onMounted(() => {
  keyEmitter.on("Escape", onEscape);
});
onBeforeUnmount(() => {
  keyEmitter.on("Escape", onEscape);
});

type ValueKey = string; // can be `split("|") for the source and value
type PathValueData = Record<ValueKey, ComponentsWithValue>;
type ComponentsDesc = Array<{ schemaName: string; componentName: string }>;
type ComponentsWithValue = {
  isSecret: boolean;
  components: ComponentsDesc;
};
const valuesByAvPath = computed(() => {
  const groupedVals = {} as Record<AttributePath, PathValueData>;
  avTrees.value
    .map((t) => t.data)
    .filter(nonNullable)
    .forEach((d) => {
      Object.values(d.attributeValues).forEach((av) => {
        let values = groupedVals[av.path];
        if (!values) {
          values = {};
          groupedVals[av.path] = values;
        }

        const source = (av.externalSources || [])[0];
        const v =
          typeof av.value === "object" ? JSON.stringify(av.value) : av.value;
        const valKey: ValueKey = `${source?.componentName}|${source?.path}|${v}`;
        let components = values[valKey];
        if (!components) {
          components = {
            isSecret: source?.isSecret || false,
            components: [] as ComponentsDesc,
          } as ComponentsWithValue;
          values[valKey] = components;
        }
        components.components.push({
          schemaName: d.schemaName,
          componentName: d.componentName,
        });
      });
    });

  return groupedVals;
});

const selectedPathName = ref<string>("");
const selectedPathPath = ref<string>("");
const pathValueData = ref<PathValueData | undefined>();
const historyValueData = ref<PathValueData | undefined>();
// find the path based data for display
attributeEmitter.on("selectedPath", ({ path, name }) => {
  selectedPathName.value = name;
  selectedPathPath.value = path;
  const vals = valuesByAvPath.value[path as AttributePath];
  pathValueData.value = vals;
  if (showHistory[path as AttributePath])
    historyValueData.value = history[path as AttributePath];
});

watch(valuesByAvPath, () => {
  if (selectedPathPath.value) {
    const vals = valuesByAvPath.value[selectedPathPath.value as AttributePath];
    pathValueData.value = vals;
  }
});

const emit = defineEmits<{
  (e: "close"): void;
  (e: "deselect", index: number): void;
}>();
</script>

<style lang="less" scoped>
// mostly copied from `ExploreGrid`, we should extract these into common, non-scoped, styles if we're going to continue down this path
.bulk-header {
  padding: 0 0.5rem 0 0.5rem;
  height: 2.75rem;
  border-top: 1px solid #d4d4d8; /* neutral-300 */
  border-bottom: 1px solid #d4d4d8; /* neutral-300 */
}
</style>
