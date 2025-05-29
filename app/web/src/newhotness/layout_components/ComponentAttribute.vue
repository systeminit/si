<template>
  <li :class="clsx('flex flex-col', !showingChildren && 'mb-[-1px]')">
    <template v-if="showingChildren">
      <AttributeChildLayout>
        <template #header>
          <div class="flex flex-row items-center gap-2xs">
            <div>{{ displayName }}</div>
            <IconButton
              v-if="props.attributeTree.isBuildable"
              icon="trash"
              size="sm"
              iconTone="destructive"
              iconIdleTone="shade"
              loadingIcon="loader"
              :loading="bifrostingTrash"
              @click="remove"
            />
          </div>
        </template>
        <ul v-if="!bifrostingTrash">
          <ComponentAttribute
            v-for="child in attributeTree.children"
            :key="child.id"
            :component="component"
            :attributeTree="child"
            @save="
              (path, id, value, connectingComponentId) =>
                emit('save', path, id, value, connectingComponentId)
            "
            @delete="(path, id) => emit('delete', path, id)"
          />
        </ul>
        <div
          v-if="isBuildable"
          class="grid grid-cols-2 items-center gap-xs relative"
        >
          <div class="p-xs">
            <VButton
              class="font-normal"
              tone="shade"
              variant="ghost"
              size="sm"
              :loading="addApi.bifrosting.value"
              :disabled="addApi.bifrosting.value"
              loadingIcon="loader"
              :tabindex="-1"
              @click="() => add()"
            >
              + add
            </VButton>
          </div>
          <template v-if="props.attributeTree.prop?.kind === 'map'">
            <keyForm.Field name="key">
              <template #default="{ field }">
                <input
                  :class="
                    clsx(
                      'block ml-auto border w-full h-lg font-mono text-sm',
                      themeClasses(
                        'text-black bg-white border-neutral-400 disabled:bg-neutral-200',
                        'text-white bg-black border-neutral-600 disabled:bg-neutral-900',
                      ),
                    )
                  "
                  type="text"
                  placeholder="Enter a key"
                  :value="field.state.value"
                  :disabled="wForm.bifrosting.value"
                  @input="(e) => field.handleChange((e.target as HTMLInputElement).value)"
                  @keypress.enter.stop.prevent="saveKeyIfFormValid"
                />
              </template>
            </keyForm.Field>
          </template>
        </div>
      </AttributeChildLayout>
    </template>
    <template v-else-if="!props.attributeTree.prop?.hidden">
      <AttributeInput
        :displayName="displayName"
        :attributeValueId="props.attributeTree.attributeValue.id"
        :path="props.attributeTree.attributeValue.path ?? ''"
        :kind="props.attributeTree.prop?.widgetKind"
        :prop="props.attributeTree.prop"
        :value="props.attributeTree.attributeValue.value?.toString() ?? ''"
        :canDelete="props.attributeTree.isBuildable"
        :isSetByConnection="
          props.attributeTree.attributeValue.isFromExternalSource
        "
        :isArray="props.attributeTree.prop?.kind === 'array'"
        :isMap="props.attributeTree.prop?.kind === 'map'"
        @save="
          (path, id, value, connectingComponentId) =>
            emit('save', path, id, value, connectingComponentId)
        "
        @delete="(path, id) => emit('delete', path, id)"
        @add="add"
      />
    </template>
  </li>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { VButton, IconButton, themeClasses } from "@si/vue-lib/design-system";
import { useRoute } from "vue-router";
import clsx from "clsx";
import { BifrostComponent } from "@/workers/types/entity_kind_types";
import AttributeChildLayout from "./AttributeChildLayout.vue";
import AttributeInput from "./AttributeInput.vue";
import { AttrTree } from "../AttributePanel.vue";
import { useApi, routes, componentTypes } from "../api_composables";
import { useWatchedForm } from "../logic_composables/watched_form";

const props = defineProps<{
  component: BifrostComponent;
  attributeTree: AttrTree;
}>();

const hasChildren = computed(() => {
  if (!props.attributeTree.prop) return false;
  switch (props.attributeTree.prop.kind) {
    case "array":
    case "map":
    case "object":
      return true;
    default:
      return false;
  }
});

const isBuildable = computed(() =>
  ["array", "map"].includes(props.attributeTree.prop?.kind ?? ""),
);

const displayName = computed(() => {
  if (props.attributeTree.attributeValue.key)
    return props.attributeTree.attributeValue.key;
  else return props.attributeTree.prop?.name || "XXX";
});

const addApi = useApi();

const add = async (key?: string) => {
  if (props.attributeTree.prop?.kind === "map") {
    if (!key) {
      saveKeyIfFormValid();
      return;
    } else {
      keyForm.setFieldValue("key", key, {});
      await saveKey();
      return;
    }
  }

  const call = addApi.endpoint<{ success: boolean }>(
    routes.UpdateComponentAttributes,
    { id: props.component.id },
  );
  addApi.setWatchFn(
    // once the children count updates, we can stop spinning
    () => props.attributeTree.children.length,
  );
  const payload: componentTypes.UpdateComponentAttributesArgs = {};
  const path =
    props.attributeTree.prop?.path
      .replace("root", "")
      .replaceAll("\u000b", "/") ?? ""; // endpoint doesn't want it

  // Do I send `{}` for array of map/object or "" for array of string?
  // Answer by looking at my prop child
  const propTree = props.component.schemaVariant.propTree;
  const childProp =
    propTree.props[
      propTree.treeInfo[props.attributeTree.prop?.id ?? ""]?.children[0] ?? ""
    ];
  if (childProp?.kind === "object") payload[`${path}/-`] = {};
  else payload[`${path}/-`] = "";
  const { req, newChangeSetId } =
    await call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
  if (addApi.ok(req) && newChangeSetId) {
    addApi.navigateToNewChangeSet(
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

const route = useRoute();
const wForm = useWatchedForm<{ key: string }>(
  `component.av.key.${props.attributeTree.prop?.id}`,
);
const keyData = ref({ key: "" });
const keyApi = useApi();
const keyForm = wForm.newForm({
  data: keyData,
  onSubmit: async ({ value }) => {
    const call = keyApi.endpoint<{ success: boolean }>(
      routes.UpdateComponentAttributes,
      { id: props.component.id },
    );
    const payload: componentTypes.UpdateComponentAttributesArgs = {};
    const path =
      props.attributeTree.prop?.path
        .replace("root", "")
        .replaceAll("\u000b", "/") ?? ""; // endpoint doesn't want it
    payload[`${path}/${value.key}`] = "";
    const { req, newChangeSetId } =
      await call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
    if (newChangeSetId && keyApi.ok(req)) {
      keyApi.navigateToNewChangeSet(
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
  },
});

const saveKeyIfFormValid = async () => {
  if (keyForm.fieldInfo.key.instance?.state.meta.isDirty) {
    if (!keyForm.baseStore.state.isSubmitted) {
      await saveKey();
    }
  }
};

const saveKey = async () => {
  await keyForm.handleSubmit();

  // this gets us the bifrosting spinner
  watch(
    () => props.attributeTree.children.length,
    () => {
      // we need nextTick here because otherwise the reset doesn't work
      // not 100% sure why, but probably has something to do with the Vue template
      // rerendering between the if/else main sections - only one has the form displaying!
      nextTick(() => keyForm.reset(keyData.value));
    },
    { once: true },
  );
};

// NOTE: we never need to unset this, because this whole node will
// be removed from the DOM once the update comes over the wire
const bifrostingTrash = ref(false);
const remove = () => {
  if (props.attributeTree.attributeValue.path) {
    emit(
      "delete",
      props.attributeTree.attributeValue.path,
      props.attributeTree.attributeValue.id,
    );
    bifrostingTrash.value = true;
  }
};

const emit = defineEmits<{
  (
    e: "save",
    path: string,
    id: string,
    value: string,
    connectingComponentId?: string,
  ): void;
  (e: "delete", path: string, id: string): void;
}>();

const showingChildren = computed(
  () => hasChildren.value && props.attributeTree.children.length > 0,
);
</script>
