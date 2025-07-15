<template>
  <div v-if="attributeTree && root" class="p-xs flex flex-col gap-xs">
    <div
      v-if="importFunc && !component.toDelete"
      class="grid grid-cols-2 gap-2xs relative text-sm h-lg"
    >
      <div class="flex flex-row items-center gap-2xs">
        <TruncateWithTooltip>{{
          importing ? "Importing Attributes" : "Import"
        }}</TruncateWithTooltip>
      </div>
      <input
        ref="importInputRef"
        v-model="resourceIdFormValue"
        :class="
          clsx(
            'block w-full h-lg p-xs ml-auto text-sm border font-mono',
            themeClasses(
              'text-shade-100 bg-shade-0 border-neutral-400',
              'text-shade-0 bg-shade-100 border-neutral-600',
            ),
          )
        "
        type="text"
        placeholder="Resource Id"
        @keydown.enter="doImport"
      />
      <Icon
        v-if="importing || bifrostingResourceId"
        class="absolute right-[54px] top-xs pointer-events-none"
        name="loader"
        size="sm"
        tone="action"
      />
      <TextPill
        class="absolute text-xs right-xs top-[7px] cursor-default"
        variant="key2"
        :class="
          clsx(!(importing || bifrostingResourceId) && 'hover:cursor-pointer')
        "
        @click.prevent="doImport"
      >
        Enter
      </TextPill>
    </div>

    <div class="grid grid-cols-2 items-center gap-2xs text-sm h-lg">
      <nameForm.Field
        :validators="{
          onChange: required,
          onBlur: required,
        }"
        name="name"
      >
        <template #default="{ field }">
          <div>Name</div>
          <div
            v-if="component.toDelete"
            ref="nameInputRef"
            v-tooltip="{
              content: 'Unable to edit this value.',
              placement: 'left',
            }"
            :class="
              clsx(
                'h-lg p-xs text-sm border font-mono',
                'focus:outline-none focus:ring-0 focus:z-10',
                'cursor-not-allowed focus:outline-none focus:z-10',
                themeClasses(
                  'bg-neutral-100 text-neutral-600 border-neutral-400 focus:border-action-500',
                  'bg-neutral-900 text-neutral-400 border-neutral-600 focus:border-action-300',
                ),
              )
            "
            tabindex="0"
            @keydown.tab.stop.prevent="onNameInputTab"
          >
            {{ field.state.value }}
          </div>
          <input
            v-else
            ref="nameInputRef"
            :value="field.state.value"
            :placeholder="namePlaceholder"
            :class="
              clsx(
                'h-lg p-xs text-sm border font-mono cursor-text',
                'focus:outline-none focus:ring-0 focus:z-10',
                themeClasses(
                  'text-shade-100 bg-white border-neutral-400 focus:border-action-500',
                  'text-shade-0 bg-black border-neutral-600 focus:border-action-300',
                ),
              )
            "
            tabindex="0"
            type="text"
            @blur="blurNameInput"
            @input="(e: Event) => { handleNameInput(e, field); }"
            @keydown.enter.stop.prevent="blurNameInput"
            @keydown.esc.stop.prevent="resetNameInput"
            @keydown.tab.stop.prevent="onNameInputTab"
          />
        </template>
      </nameForm.Field>
    </div>

    <div>
      <!-- TODO(Wendy) - this doesn't work on the secrets tree yet -->
      <SiSearch
        ref="searchRef"
        v-model="q"
        placeholder="filter attributes..."
        :tabIndex="0"
        :borderBottom="false"
        @keydown.tab="onSearchInputTab"
      />
    </div>
    <div
      v-if="'children' in filtered.tree && filtered.tree.children.length > 0"
    >
      <h3
        :class="
          clsx(
            'p-xs border-l',
            themeClasses(
              'bg-neutral-200 border-neutral-200',
              'bg-neutral-800 border-neutral-800',
            ),
          )
        "
      >
        domain
      </h3>
      <!-- this is _really_ a type guard for "i am not an empty object" -->
      <ul
        :class="
          clsx(
            'border-l border-r',
            themeClasses('border-neutral-200', 'border-neutral-800'),
          )
        "
      >
        <ComponentAttribute
          v-for="child in filtered.tree.children"
          :key="child.id"
          :component="component"
          :attributeTree="child"
          @save="save"
          @delete="remove"
          @remove-subscription="removeSubscription"
        />
      </ul>
      <div
        :class="
          clsx(
            'w-full border-b',
            themeClasses('border-neutral-200', 'border-neutral-800'),
          )
        "
      />
    </div>
    <div v-if="secrets && secrets.children.length > 0">
      <h3
        :class="
          clsx(
            'p-xs border-l',
            themeClasses(
              'bg-neutral-200 border-neutral-200',
              'bg-neutral-800 border-neutral-800',
            ),
          )
        "
      >
        secrets
      </h3>
      <ul
        v-if="'children' in secrets"
        :class="
          clsx(
            'border-l',
            themeClasses('border-neutral-200', 'border-neutral-800'),
          )
        "
      >
        <ComponentSecretAttribute
          v-for="secret in secrets.children"
          :key="secret.id"
          :component="component"
          :attributeTree="secret"
        />
      </ul>
      <div
        :class="
          clsx(
            'w-full border-b',
            themeClasses('border-neutral-200', 'border-neutral-800'),
          )
        "
      />
    </div>
  </div>
  <EmptyState v-else text="No attributes to display" icon="code-circle" />
</template>

<script lang="ts" setup>
import {
  computed,
  onBeforeUnmount,
  onMounted,
  reactive,
  ref,
  watch,
} from "vue";
import { Fzf } from "fzf";
import { useRoute } from "vue-router";
import {
  Icon,
  SiSearch,
  themeClasses,
  TruncateWithTooltip,
  TextPill,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import * as _ from "lodash-es";
import { useMutation, useQueryClient } from "@tanstack/vue-query";
import {
  AttributeTree,
  AttributeValue,
  BifrostComponent,
  EntityKind,
  MgmtFunction,
  Prop,
  Secret,
} from "@/workers/types/entity_kind_types";
import { PropKind } from "@/api/sdf/dal/prop";
import { useMakeKey } from "@/store/realtime/heimdall";
import { FuncRun } from "@/newhotness/api_composables/func_run";
import { AttributePath, ComponentId } from "@/api/sdf/dal/component";
import { componentTypes, routes, useApi } from "./api_composables";
import ComponentAttribute from "./layout_components/ComponentAttribute.vue";
import { keyEmitter } from "./logic_composables/emitters";
import ComponentSecretAttribute from "./layout_components/ComponentSecretAttribute.vue";
import { useWatchedForm } from "./logic_composables/watched_form";
import { NameFormData } from "./ComponentDetails.vue";
import EmptyState from "./EmptyState.vue";
import { findAttributeValueInTree } from "./util";

const q = ref("");

const props = defineProps<{
  component: BifrostComponent;
  attributeTree?: AttributeTree;
  importFunc?: MgmtFunction;
  importFuncRun?: FuncRun;
}>();

export interface AttrTree {
  id: string;
  children: AttrTree[];
  parent?: string;
  prop?: Prop;
  secret?: Secret;
  attributeValue: AttributeValue;
  isBuildable: boolean; // is my parent an array or map?
}

const makeAvTree = (
  data: AttributeTree,
  avId: string,
  isBuildable: boolean,
  parent?: string,
): AttrTree => {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const av = data.attributeValues[avId]!;
  const prop = av.propId ? data.props[av.propId] : undefined;
  const secret = av.secret ?? undefined;
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const childrenIds = data.treeInfo[avId]!.children;
  const children = childrenIds.map((id) =>
    makeAvTree(data, id, ["array", "map"].includes(prop?.kind ?? ""), avId),
  );
  const tree: AttrTree = {
    id: avId,
    children,
    parent,
    attributeValue: av,
    prop,
    secret,
    isBuildable,
  };
  return tree;
};

const root = computed<AttrTree>(() => {
  const empty = {
    id: "",
    children: [] as AttrTree[],
    attributeValue: {} as AttributeValue,
    isBuildable: false,
  };
  const raw = props.attributeTree;
  if (!raw) return empty;

  // find the root node in the tree, the only one with parent null
  const rootId = Object.keys(raw.treeInfo).find((avId) => {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const av = raw.treeInfo[avId]!;
    if (!av.parent) return true;
    return false;
  });
  if (!rootId) return empty;

  const tree = makeAvTree(raw, rootId, false);
  return tree;
});

const domain = computed(() =>
  root.value?.children.find((c) => c.prop?.name === "domain"),
);

const secrets = computed(() =>
  root.value?.children.find((c) => c.prop?.name === "secrets"),
);

const filtered = reactive<{ tree: AttrTree | object }>({
  tree: {},
});

watch(
  () => [q.value, domain.value],
  () => {
    if (!q.value) {
      filtered.tree = domain.value ?? {};
      return;
    }
    if (!domain.value) {
      filtered.tree = {};
      return;
    }

    // we need to access attrs by id
    const map: Record<string, AttrTree> = {};
    map[domain.value.id] = domain.value;
    const walking = [...domain.value.children];
    // walk all the children and find if they match
    while (walking.length > 0) {
      const attr = walking.shift();
      if (!attr) break;
      map[attr.id] = attr;
      walking.push(...attr.children);
    }

    const fzf = new Fzf(Object.values(map), {
      casing: "case-insensitive",
      selector: (p) =>
        `${p.id} ${p.prop?.name} ${p.prop?.path} ${p.attributeValue.key} ${p.attributeValue.value}`,
    });

    const results = fzf.find(q.value);
    // Maybe we want to get rid of low scoring options (via std dev)?
    const matches: AttrTree[] = results.map((fz) => fz.item);

    // get new instances of all the objects with empty children arrays
    const parentsWithoutChildren = Object.values(map)
      .map((attr) => {
        return {
          ...attr,
          children: [],
        };
      })
      .reduce((map, attr) => {
        map[attr.id] = attr;
        return map;
      }, {} as Record<string, AttrTree>);

    const matchesAsTree: Record<string, AttrTree> = {};
    // work backwards from the leaf node, filling in their parents children arrays
    // make sure there are no dupes b/c matches will give us dupes
    matches.forEach((attr) => {
      const parents = [attr.parent];
      let prevPid: string | undefined;
      while (parents.length > 0) {
        const pId = parents.shift();
        if (!pId) throw new Error("no pid");
        let p: AttrTree | undefined;
        p = matchesAsTree[pId];
        if (!p) p = parentsWithoutChildren[pId];
        if (p) {
          if (prevPid) {
            const lastParent = matchesAsTree[prevPid];
            if (lastParent && !p.children.some((c) => c.id === lastParent.id))
              p.children.push(lastParent);
          } else if (!p.children.some((c) => c.id === attr.id))
            p.children.push(attr);

          matchesAsTree[p.id] = p;

          if (p.parent && p.id !== domain.value?.id)
            // dont traverse past domain
            parents.push(p.parent);
        }
        prevPid = pId;
      }
    });

    // all roads lead back to domain
    const newDomain = matchesAsTree[domain.value.id];
    filtered.tree = newDomain ?? {};
  },
  { immediate: true },
);

const route = useRoute();

const saveApi = useApi();

const save = async (
  path: AttributePath,
  value: string,
  propKind: PropKind,
  connectingComponentId?: ComponentId,
) => {
  const call = saveApi.endpoint<{ success: boolean }>(
    routes.UpdateComponentAttributes,
    { id: props.component.id },
  );

  // TODO - Paul there's a better way to handle this for sure!
  let coercedVal: string | boolean | number | null = value;
  if (value === "") {
    // For now we don't allow a user to enter an empty string
    // passing an empty string is effectively an unset operation
    coercedVal = null;
  } else if (propKind === PropKind.Boolean) {
    coercedVal = value.toLowerCase() === "true" || value === "1";
  } else if (propKind === PropKind.Integer) {
    coercedVal = Math.trunc(Number(value));
  } else if (propKind === PropKind.Float) {
    coercedVal = Number(value);
  }

  const payload: componentTypes.UpdateComponentAttributesArgs = {};
  payload[path] = coercedVal;
  if (connectingComponentId) {
    payload[path] = {
      $source: { component: connectingComponentId, path: coercedVal },
    };
  }

  const { req, newChangeSetId } =
    await call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
  if (saveApi.ok(req) && newChangeSetId) {
    saveApi.navigateToNewChangeSet(
      {
        name: "new-hotness-component",
        params: {
          workspacePk: route.params.workspacePk,
          changeSetId: newChangeSetId,
          componentId: props.component.id,
        },
      },
      newChangeSetId,
    );
  }
};

const removeApi = useApi();

const remove = async (path: AttributePath) => {
  const call = removeApi.endpoint<{ success: boolean }>(
    routes.UpdateComponentAttributes,
    { id: props.component.id },
  );
  const payload: componentTypes.UpdateComponentAttributesArgs = {};
  payload[path] = { $source: null };
  const { req, newChangeSetId } =
    await call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
  if (removeApi.ok(req) && newChangeSetId) {
    removeApi.navigateToNewChangeSet(
      {
        name: "new-hotness-component",
        params: {
          workspacePk: route.params.workspacePk,
          changeSetId: newChangeSetId,
          componentId: props.component.id,
        },
      },
      newChangeSetId,
    );
  }
};

const removeSubscriptionApi = useApi();
const queryClient = useQueryClient();
const makeKey = useMakeKey();

const removeSubscriptionMutation = useMutation({
  mutationFn: async (path: AttributePath) => {
    const call = removeSubscriptionApi.endpoint<{ success: boolean }>(
      routes.UpdateComponentAttributes,
      { id: props.component.id },
    );

    const payload: componentTypes.UpdateComponentAttributesArgs = {};
    payload[path] = {
      $source: null,
    };

    const { req, newChangeSetId } =
      await call.put<componentTypes.UpdateComponentAttributesArgs>(payload);

    if (removeSubscriptionApi.ok(req) && newChangeSetId) {
      removeSubscriptionApi.navigateToNewChangeSet(
        {
          name: "new-hotness-component",
          params: {
            workspacePk: route.params.workspacePk,
            changeSetId: newChangeSetId,
            componentId: props.component.id,
          },
        },
        newChangeSetId,
      );
    }

    return { req, newChangeSetId };
  },
  onMutate: async (path: AttributePath) => {
    const queryKey = makeKey(EntityKind.AttributeTree, props.component.id);

    const previousData = queryClient.getQueryData<AttributeTree>(
      queryKey.value,
    );

    queryClient.setQueryData(
      queryKey.value,
      (cachedData: AttributeTree | undefined) => {
        if (!cachedData) {
          return cachedData;
        }

        const found = findAttributeValueInTree(cachedData, path);
        if (!found) {
          return cachedData;
        }

        const updatedData = { ...cachedData };
        const updatedFound = findAttributeValueInTree(updatedData, path);
        if (updatedFound) {
          updatedFound.attributeValue.externalSources = undefined;
          updatedFound.attributeValue.value = null;
        }

        return updatedData;
      },
    );

    return { previousData };
  },
  onError: (error, path, context) => {
    if (context?.previousData) {
      const queryKey = makeKey(EntityKind.AttributeTree, props.component.id);
      queryClient.setQueryData(queryKey.value, context.previousData);
    }
  },
});

const removeSubscription = async (path: AttributePath) => {
  removeSubscriptionMutation.mutate(path);
};

const nameInputRef = ref<HTMLInputElement | HTMLDivElement>();
const searchRef = ref<InstanceType<typeof SiSearch>>();

// Import
const resourceIdAttr = computed(() => {
  const siTree = root.value.children.find((p) => p.prop?.name === "si");
  return siTree?.children.find((p) => p.prop?.name === "resourceId");
});

const resourceIdValue = computed(
  () => resourceIdAttr.value?.attributeValue.value ?? null,
);
const resourceIdFormValue = ref<string | undefined>();

const importInputRef = ref<HTMLInputElement>();

const bifrostingResourceId = ref(false);
const resettingResourceId = ref(false);
const saveResourceId = async () => {
  if (!resourceIdFormValue.value) {
    return;
  }

  bifrostingResourceId.value = true;

  await save("/si/resourceId", resourceIdFormValue.value, PropKind.String);
};

watch(
  resourceIdFormValue,
  _.debounce(
    () => {
      if (resettingResourceId.value) {
        resettingResourceId.value = false;
        return;
      }
      saveResourceId();
    },
    500,
    { leading: true },
  ),
);

watch([resourceIdValue], () => {
  if (resourceIdFormValue.value === resourceIdValue.value) {
    bifrostingResourceId.value = false;
  }
});

const runMgmtFuncApi = useApi();

const doImport = async () => {
  if (bifrostingResourceId.value) {
    return;
  }

  const func = props.importFunc;
  if (!func) return;

  spawningFunc.value = true;

  const call = runMgmtFuncApi.endpoint<{ success: boolean }>(
    routes.MgmtFuncRun,
    {
      prototypeId: func.id,
      componentId: props.component.id,
      viewId: "DEFAULT",
    },
  );

  const { req, newChangeSetId } =
    await call.post<componentTypes.UpdateComponentAttributesArgs>({});

  setTimeout(() => {
    spawningFunc.value = false;
  }, 2000);

  if (runMgmtFuncApi.ok(req) && newChangeSetId) {
    runMgmtFuncApi.navigateToNewChangeSet(
      {
        name: "new-hotness-component",
        params: {
          workspacePk: route.params.workspacePk,
          changeSetId: newChangeSetId,
          componentId: props.component.id,
        },
      },
      newChangeSetId,
    );
  }
};

const spawningFunc = ref(false);
const importing = computed(
  () =>
    spawningFunc.value ||
    ["Created", "Dispatched", "Running", "Postprocessing"].includes(
      props.importFuncRun?.state ?? "",
    ),
);

onMounted(() => {
  resettingResourceId.value = true;
  resourceIdFormValue.value = resourceIdValue.value ?? undefined;
  keyEmitter.on("Tab", (e) => {
    e.preventDefault();
    focusNameInput();
  });
  if (nameIsDefault.value) focusNameInput();
  else focusSearch();
});

const focusSearch = () => {
  searchRef.value?.focusSearch();
};

const focusNameInput = () => {
  nameInputRef.value?.focus();
};

onBeforeUnmount(() => {
  keyEmitter.off("Tab");
});

const onNameInputTab = (e: KeyboardEvent) => {
  if (e.shiftKey) {
    e.preventDefault();
    const focusable = Array.from(
      document.querySelectorAll('[tabindex="0"]'),
    ) as HTMLElement[];
    if (focusable) {
      focusable[focusable.length - 1]?.focus();
    }
  } else {
    nameInputRef.value?.blur();
    focusSearch();
  }
};

const onSearchInputTab = (e: KeyboardEvent) => {
  if (e.shiftKey) {
    e.preventDefault();
    focusNameInput();
  }
};

const api = useApi();

const required = ({ value }: { value: string | undefined }) => {
  const len = value?.trim().length ?? 0;
  return len > 0 ? undefined : "Name required";
};

const resetNameInput = () => {
  if (nameForm.state.isSubmitted || nameForm.state.isDirty) {
    nameForm.reset(nameFormData.value);
  }
  nameInputRef.value?.blur();
};

const blurNameInput = () => {
  if (
    nameForm.fieldInfo.name.instance?.state.meta.isDirty &&
    !props.component.toDelete
  ) {
    // don't double submit if you were `select()'d'`
    if (!nameForm.baseStore.state.isSubmitted) nameForm.handleSubmit();
  } else {
    resetNameInput();
  }
};
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const handleNameInput = (e: Event, field: any) => {
  if (props.component.toDelete) return;
  field.handleChange((e.target as HTMLInputElement).value);
};

const nameIsDefault = computed(() => props.component.name.startsWith("si-"));
const namePlaceholder = computed(() =>
  nameIsDefault.value
    ? props.component.name
    : "You must give the component a name!",
);

const wForm = useWatchedForm<NameFormData>(
  `component.name.${props.component.id}`,
);

const nameFormData = computed<NameFormData>(() => {
  return { name: nameIsDefault.value ? "" : props.component.name };
});

const nameForm = wForm.newForm({
  data: nameFormData,
  onSubmit: async ({ value }) => {
    const name = value.name;
    // i wish the validator narrowed this type to always be a string
    if (name) {
      const id = props.component.id;
      const call = api.endpoint(routes.UpdateComponentName, { id });
      const { req, newChangeSetId } =
        await call.put<componentTypes.UpdateComponentNameArgs>({
          name,
        });
      if (newChangeSetId && api.ok(req)) {
        api.navigateToNewChangeSet(
          {
            name: "new-hotness-component",
            params: {
              workspacePk: route.params.workspacePk,
              changeSetId: newChangeSetId,
              componentId: props.component.id,
            },
          },
          newChangeSetId,
        );
      }
    }
  },
});
</script>
