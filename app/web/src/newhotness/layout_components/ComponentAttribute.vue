<template>
  <li :class="clsx('flex flex-col', !hasChildren && 'mb-[-1px]')">
    <template v-if="hasChildren">
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
        <ul v-if="attributeTree.children.length > 0 && !bifrostingTrash">
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
        <template v-if="isBuildable">
          <Icon
            v-if="wForm.bifrosting.value"
            name="loader"
            size="sm"
            tone="action"
          />
          <template v-if="showKey">
            <keyForm.Field name="key">
              <template #default="{ field }">
                <input
                  class="block w-72 text-white bg-black border border-neutral-300 disabled:bg-neutral-900"
                  type="text"
                  :value="field.state.value"
                  :disabled="wForm.bifrosting.value"
                  @input="(e) => field.handleChange((e.target as HTMLInputElement).value)"
                  @blur="saveKey"
                  @keypress.enter.stop.prevent="saveKey"
                />
              </template>
            </keyForm.Field>
          </template>
          <div class="p-xs">
            <!-- TODO(Wendy) - could not figure out how to get Tab to work for these buttons! -->
            <VButton
              class="font-normal"
              tone="shade"
              variant="ghost"
              size="sm"
              :loading="addApi.bifrosting.value"
              :disabled="addApi.bifrosting.value"
              loadingIcon="loader"
              :tabindex="-1"
              @click="add"
            >
              + add
            </VButton>
          </div>
        </template>
      </AttributeChildLayout>
    </template>
    <template v-else>
      <AttributeInput
        :displayName="displayName"
        :attributeValueId="props.attributeTree.attributeValue.id"
        :path="props.attributeTree.attributeValue.path ?? ''"
        :kind="props.attributeTree.prop?.widgetKind"
        :prop="props.attributeTree.prop"
        :value="props.attributeTree.attributeValue.value?.toString() ?? ''"
        :canDelete="props.attributeTree.isBuildable"
        @save="
          (path, id, value, connectingComponentId) =>
            emit('save', path, id, value, connectingComponentId)
        "
        @delete="(path, id) => emit('delete', path, id)"
      />
    </template>
  </li>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { VButton, IconButton, Icon } from "@si/vue-lib/design-system";
import { useRoute, useRouter } from "vue-router";
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

const add = async () => {
  if (props.attributeTree.prop?.kind === "map") {
    showKey.value = true;
    return;
  }

  addApi.setWatchFn(
    // once the children count updates, we can stop spinning
    () => props.attributeTree.children.length,
  );

  const call = addApi.endpoint<{ success: boolean }>(
    routes.UpdateComponentAttributes,
    { id: props.component.id },
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

const router = useRouter();
const route = useRoute();
const showKey = ref(false);
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
});

const saveKey = async () => {
  if (keyForm.fieldInfo.key.instance?.state.meta.isDirty) {
    if (!keyForm.baseStore.state.isSubmitted) {
      await keyForm.handleSubmit();
      showKey.value = false;
      // this gets us the bifrosting spinner
      watch(
        () => props.attributeTree.children.length,
        () => {
          keyData.value = { key: "" };
          keyForm.reset(keyData.value);
        },
        { once: true },
      );
    }
  }
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
</script>
