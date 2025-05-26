<template>
  <div v-if="root">
    <div class="py-xs">
      <SiSearch
        ref="searchRef"
        v-model="q"
        placeholder="filter attributes..."
        :tabIndex="0"
        @keydown.tab="onTab"
      />
    </div>
    <h3
      :class="
        clsx('p-xs border-l', themeClasses('bg-neutral-200', 'bg-neutral-800'))
      "
    >
      domain
    </h3>
    <!-- this is _really_ a type guard for "i am not an empty object" -->
    <ul v-if="'children' in filtered.tree" class="border-l">
      <ComponentAttribute
        v-for="child in filtered.tree.children"
        :key="child.id"
        :component="component"
        :attributeTree="child"
        @save="save"
        @delete="remove"
      />
    </ul>
    <div v-else>Oh no, no attributes!</div>
    <template v-if="secret">
      <h3 class="bg-neutral-800 p-xs border-l-2">secrets</h3>
      <ul>
        <li class="flex flex-col">
          <label class="pl-xs flex flex-row items-center relative">
            <span>{{ secret.prop?.name }}</span>
            <input
              class="block w-72 ml-auto text-white bg-black border-2 border-neutral-300 disabled:bg-neutral-900"
              type="text"
              disabled
              :value="secret.secret ? `${secret.secret.name}` : ''"
            />
          </label>
        </li>
      </ul>
      <template v-if="component.isSecretDefining">
        <div class="m-xs p-xs border-2">
          <ul class="flex flex-col">
            <template
              v-for="fieldname in Object.keys(secretFormData)"
              :key="fieldname"
            >
              <li class="mb-2xs flex flex-row items-center">
                <span>{{ fieldname }}</span>
                <secretForm.Field :name="fieldname">
                  <template #default="{ field }">
                    <input
                      :class="
                        clsx(
                          'block w-64 ml-auto text-white bg-black border-2 border-neutral-300 disabled:bg-neutral-900',
                          field.state.meta.errors.length > 0 &&
                            'border-destructive-500',
                        )
                      "
                      type="text"
                      :value="field.state.value"
                      @input="(e) => field.handleChange((e.target as HTMLInputElement).value)"
                    />
                  </template>
                </secretForm.Field>
              </li>
            </template>
            <VButton
              :label="secret.secret ? 'Replace Secret' : 'Add Secret'"
              :loading="wForm.bifrosting.value"
              loadingText="Saving Secret"
              tone="action"
              :disabled="!secretForm.state.canSubmit"
              @click="() => secretForm.handleSubmit()"
            />
          </ul>
        </div>
      </template>
    </template>
  </div>
</template>

<script lang="ts" setup>
import { computed, reactive, ref, watch } from "vue";
import { Fzf } from "fzf";
import { useRoute, useRouter } from "vue-router";
import { SiSearch, themeClasses, VButton } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useQuery } from "@tanstack/vue-query";
import {
  AttributeTree,
  BifrostComponent,
  Prop,
  AttributeValue,
  EddaSecret,
  BifrostSecretDefinitionList,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import { PropertyEditorPropWidgetKindSecret } from "@/api/sdf/dal/property_editor";
import { encryptMessage } from "@/utils/messageEncryption";
import { useApi, routes, componentTypes } from "./api_composables";
import ComponentAttribute from "./layout_components/ComponentAttribute.vue";
import { useWatchedForm } from "./logic_composables/watched_form";
import { keyEmitter } from "./logic_composables/emitters";

const q = ref("");

const props = defineProps<{
  component: BifrostComponent;
}>();

export interface AttrTree {
  id: string;
  children: AttrTree[];
  parent?: string;
  prop?: Prop;
  secret?: EddaSecret;
  attributeValue: AttributeValue;
  isBuildable: boolean; // is my parent an array or map?
}

const makeArgs = useMakeArgs();
const makeKey = useMakeKey();

const secretFormData = ref<Record<string, string>>({});
// NOTE: this is pretty tortured and will change
const secretQuery = useQuery({
  queryKey: makeKey(EntityKind.SecretDefinitionList),
  enabled: props.component.isSecretDefining,
  queryFn: async () => {
    const data = (await bifrost(makeArgs(EntityKind.SecretDefinitionList))) as
      | BifrostSecretDefinitionList
      | -1;
    if (data === -1) return undefined;
    let label = "";
    if (secret.value?.prop?.widgetKind) {
      const kind = secret.value.prop.widgetKind;
      if ("secret" in kind) {
        const options = (kind.secret as PropertyEditorPropWidgetKindSecret)
          .options;
        const opt = options[0];
        if (opt) label = opt.value;
      }
    }
    const d = data.secretDefinitions.find((d) => d.label === label);
    if (d) {
      const form = d.formData
        .flatMap((row) => row.name)
        .reduce((obj, name) => {
          obj[name] = "";
          return obj;
        }, {} as Record<string, string>);
      secretFormData.value = { Name: "", ...form };
    } else secretFormData.value = {};

    secretForm.reset(secretFormData.value);
    return d;
  },
});

const makeAvTree = (
  data: AttributeTree,
  avId: string,
  isBuildable: boolean,
  parent?: string,
): AttrTree => {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const av = data.attributeValues[avId]!;
  const prop = av.propId ? data.props[av.propId] : undefined;
  const secret = av.secretId ? data.secrets[av.secretId] : undefined;
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
  const raw = props.component.attributeTree;
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

const secret = computed(
  () =>
    root.value?.children.find((c) => c.prop?.name === "secrets")?.children[0],
);

const keyApi = useApi();
const secretApi = useApi();

const wForm = useWatchedForm<Record<string, string>>(
  `component.secret.${props.component.id}`,
);
const secretForm = wForm.newForm({
  data: secretFormData,
  onSubmit: async ({ value }) => {
    const definition = secretQuery.data.value?.label;
    if (!definition) throw new Error("Secret Definition Required");
    if (!secret.value) throw new Error("Secret AV Required");
    if (!secret.value.prop) throw new Error("Secret Prop Required");
    const callApi = keyApi.endpoint<componentTypes.PublicKey>(
      routes.GetPublicKey,
      { id: props.component.id },
    );
    const resp = await callApi.get();
    const publicKey = resp.data;

    const name = value.Name ?? "";
    delete value.Name;
    const crypted = await encryptMessage(value, publicKey);

    const payload: componentTypes.CreateSecret = {
      name,
      attributeValueId: secret.value.attributeValue.id,
      propId: secret.value.prop.id,
      definition,
      crypted,
      keyPairPk: publicKey.pk,
      version: componentTypes.SecretVersion.V1,
      algorithm: componentTypes.SecretAlgorithm.Sealedbox,
    };

    const newSecret = secretApi.endpoint<{ id: string }>(routes.CreateSecret, {
      id: props.component.id,
    });
    const { req, newChangeSetId } =
      await newSecret.post<componentTypes.CreateSecret>(payload);
    if (secretApi.ok(req) && newChangeSetId) {
      router.push({
        name: "new-hotness-component",
        params: {
          workspacePk: route.params.workspacePk,
          changeSetId: newChangeSetId,
          componentId: props.component.id,
        },
      });
    }
  },
  validators: {
    onSubmit: ({ value }) => {
      return {
        fields: {
          Name: !value.Name ? "Name required" : undefined,
        },
      };
    },
  },
  watchFn: () => secret.value?.secret,
});

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

const api = useApi();

const save = async (
  path: string,
  _id: string,
  value: string,
  connectingComponentId?: string,
) => {
  const call = api.endpoint<{ success: boolean }>(
    routes.UpdateComponentAttributes,
    { id: props.component.id },
  );
  const payload: componentTypes.UpdateComponentAttributesArgs = {};
  path = path.replace("root", ""); // endpoint doesn't want it
  payload[path] = value;
  if (connectingComponentId) {
    payload[path] = {
      $source: { component: connectingComponentId, path: value },
    };
  }
  await call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
};

const router = useRouter();
const route = useRoute();
const remove = async (path: string, _id: string) => {
  const call = api.endpoint<{ success: boolean }>(
    routes.UpdateComponentAttributes,
    { id: props.component.id },
  );
  const payload: componentTypes.UpdateComponentAttributesArgs = {};
  path = path.replace("root", ""); // endpoint doesn't want it
  payload[path] = { $source: null };
  const { req, newChangeSetId } =
    await call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
  if (newChangeSetId && api.ok(req)) {
    router.push({
      name: "new-hotness-component",
      params: {
        workspacePk: route.params.workspacePk,
        changeSetId: newChangeSetId,
        componentId: props.component.id,
      },
    });
  }
};

const searchRef = ref<InstanceType<typeof SiSearch>>();

keyEmitter.on("Tab", (e) => {
  e.preventDefault();
  searchRef.value?.focusSearch();
});

const onTab = (e: KeyboardEvent) => {
  // This allows the user to Shift+Tab backwards to the last attribute from the fuzzy search bar
  if (e.shiftKey) {
    e.preventDefault();
    const focusable = Array.from(
      document.querySelectorAll('[tabindex="0"]'),
    ) as HTMLElement[];
    if (focusable) {
      focusable[focusable.length - 1]?.focus();
    }
  }
};
</script>
