<template>
  <li
    v-if="showingChildren || !attributeTree.prop?.hidden"
    :class="clsx('flex flex-col', !showingChildren && 'mb-[-1px]')"
  >
    <template v-if="showingChildren">
      <AttributeChildLayout>
        <template #header>
          <div
            ref="headerRef"
            :class="
              clsx(
                'flex flex-row items-center gap-2xs',
                attributeTree.isBuildable &&
                  'focus:outline-none group/attributeheader',
              )
            "
            :tabindex="attributeTree.isBuildable ? 0 : undefined"
            @keydown.tab.stop.prevent="onHeaderTab"
            @keydown.enter.stop.prevent="remove"
            @keydown.delete.stop.prevent="remove"
          >
            <div>{{ displayName }}</div>
            <IconButton
              v-if="attributeTree.isBuildable"
              v-tooltip="'Delete'"
              icon="trash"
              size="sm"
              iconTone="destructive"
              iconIdleTone="shade"
              loadingIcon="loader"
              :loading="bifrostingTrash"
              class="group-focus/attributeheader:outline group-focus/attributeheader:outline-action-500"
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
              (path, id, value, propKind, connectingComponentId) =>
                emit('save', path, id, value, propKind, connectingComponentId)
            "
            @delete="(path, id) => emit('delete', path, id)"
            @remove-subscription="
              (path, id) => emit('removeSubscription', path, id)
            "
          />
        </ul>
        <div
          v-if="isBuildable"
          class="grid grid-cols-2 items-center gap-xs relative"
        >
          <template v-if="attributeTree.prop?.kind === 'map'">
            <keyForm.Field name="key">
              <template #default="{ field }">
                <input
                  :class="
                    clsx(
                      'block ml-auto border w-full h-lg font-mono text-sm order-2',
                      'focus:outline-none focus:shadow-none focus:ring-0',
                      themeClasses(
                        'text-black bg-white border-neutral-400 disabled:bg-neutral-200 focus:border-action-500',
                        'text-white bg-black border-neutral-600 disabled:bg-neutral-900 focus:border-action-300',
                      ),
                    )
                  "
                  type="text"
                  placeholder="Enter a key"
                  tabindex="0"
                  :value="field.state.value"
                  :disabled="wForm.bifrosting.value"
                  @input="
                    (e) =>
                      field.handleChange((e.target as HTMLInputElement).value)
                  "
                  @keypress.enter.stop.prevent="saveKeyIfFormValid"
                />
              </template>
            </keyForm.Field>
          </template>
          <div class="p-xs">
            <VButton
              ref="addButtonRef"
              :class="
                clsx(
                  'font-normal',
                  themeClasses(
                    'focus:!border-action-500',
                    'focus:!border-action-300',
                  ),
                )
              "
              tone="shade"
              variant="ghost"
              size="sm"
              :loading="addApi.bifrosting.value"
              :disabled="addApi.bifrosting.value"
              loadingIcon="loader"
              :tabindex="0"
              @click="() => add()"
              @keydown.tab.stop.prevent="onAddButtonTab"
            >
              + add
            </VButton>
          </div>
        </div>
      </AttributeChildLayout>
    </template>
    <template v-else-if="!attributeTree.prop?.hidden">
      <AttributeInput
        :displayName="displayName"
        :attributeValueId="props.attributeTree.attributeValue.id"
        :path="attributeTree.attributeValue.path ?? ''"
        :kind="attributeTree.prop?.widgetKind"
        :prop="attributeTree.prop"
        :validation="attributeTree.attributeValue.validation"
        :component="component"
        :value="attributeTree.attributeValue.value?.toString() ?? ''"
        :canDelete="attributeTree.isBuildable"
        :externalSources="attributeTree.attributeValue.externalSources"
        :isArray="attributeTree.prop?.kind === 'array'"
        :isMap="attributeTree.prop?.kind === 'map'"
        @save="(...args) => emit('save', ...args)"
        @delete="(...args) => emit('delete', ...args)"
        @remove-subscription="(...args) => emit('removeSubscription', ...args)"
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
import { PropKind } from "@/api/sdf/dal/prop";
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
  watchFn: () => props.attributeTree.children.length,
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
  if (
    props.attributeTree.attributeValue.path &&
    props.attributeTree.isBuildable
  ) {
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
    propKind: PropKind,
    connectingComponentId?: string,
  ): void;
  (e: "delete", path: string, id: string): void;
  (e: "removeSubscription", path: string, id: string): void;
}>();

const showingChildren = computed(
  () => hasChildren.value && props.attributeTree.children.length > 0,
);

const headerRef = ref<HTMLDivElement>();
const addButtonRef = ref<InstanceType<typeof VButton>>();

const handleTab = (e: KeyboardEvent, currentFocus?: HTMLElement) => {
  const focusable = Array.from(
    document.querySelectorAll('[tabindex="0"]'),
  ) as HTMLElement[];

  if (!currentFocus) return;
  const index = focusable.indexOf(currentFocus);

  if (e.shiftKey) {
    nextTick(() => {
      if (currentFocus && focusable) {
        if (index > 0) {
          focusable[index - 1]?.focus();
        } else {
          focusable[focusable.length - 1]?.focus();
        }
      }
    });
  } else if (index === focusable.length - 1) {
    // When you hit the last attribute, go back to the
    // fuzzy search instead of searching the document for more things to tab to.
    e.preventDefault();
    nextTick(() => {
      focusable[0]?.focus();
    });
  } else {
    nextTick(() => {
      focusable[index + 1]?.focus();
    });
  }
};

const onHeaderTab = (e: KeyboardEvent) => {
  handleTab(e, headerRef.value);
};
const onAddButtonTab = (e: KeyboardEvent) => {
  handleTab(e, addButtonRef.value?.$el);
};
</script>
