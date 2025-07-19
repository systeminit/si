<template>
  <!-- Subscription input - appears when creating subscription -->
  <AttributeInput
    v-if="showSubscriptionInput"
    ref="subscriptionInputRef"
    :displayName="`Connect ${displayName}`"
    :path="attributeTree.attributeValue.path"
    :kind="attributeTree.prop?.widgetKind"
    :prop="attributeTree.prop"
    :validation="attributeTree.attributeValue.validation"
    :component="component"
    :value="''"
    :canDelete="false"
    :externalSources="attributeTree.attributeValue.externalSources"
    :isArray="false"
    :isMap="false"
    :forceReadOnly="false"
    @close="closeSubscriptionInput"
    @save="(...args) => emit('save', ...args)"
  />
  <li
    v-else-if="showingChildren || !attributeTree.prop?.hidden"
    :class="clsx('flex flex-col', !showingChildren && 'mb-[-1px]')"
  >
    <template v-if="showingChildren">
      <AttributeChildLayout>
        <template #header>
          <div
            ref="headerRef"
            :class="
              clsx(
                'flex flex-row items-center gap-2xs w-full',
                attributeTree.isBuildable &&
                  'focus:outline-none group/attributeheader',
              )
            "
            @keydown.tab.stop.prevent="onHeaderTab"
            @keydown.enter.stop.prevent="remove"
            @keydown.delete.stop.prevent="remove"
          >
            <div>{{ displayName }}</div>
            <div class="flex-1" />
            <div
              v-if="attributeTree.attributeValue.externalSources?.length"
              class="flex items-center gap-xs text-xs flex-shrink-0"
            >
              <span
                :class="themeClasses('text-neutral-500', 'text-neutral-400')"
              >
                Set via subscription to
              </span>
              <span class="text-purple">
                {{
                  attributeTree.attributeValue.externalSources[0]?.componentName
                }}
              </span>
              <span
                :class="themeClasses('text-neutral-600', 'text-neutral-400')"
              >
                {{
                  attributeTree.attributeValue.externalSources[0]?.path
                }}</span
              >

              <IconButton
                v-tooltip="'Remove subscription'"
                icon="x"
                size="sm"
                iconTone="destructive"
                iconIdleTone="shade"
                @click="removeSubscription"
              />
            </div>
            <VButton
              v-if="
                attributeTree.isBuildable &&
                !component.toDelete &&
                !parentHasExternalSources &&
                attributeTree.prop?.kind === 'object' &&
                !attributeTree.attributeValue.externalSources?.length
              "
              ref="connectButtonRef"
              v-tooltip="'Create subscription'"
              :tabIndex="
                attributeTree.isBuildable && !component.toDelete ? 0 : undefined
              "
              variant="ghost"
              size="xs"
              class="focus:outline focus:outline-action-500"
              @click.stop.prevent="createSubscription"
              @keydown.tab.stop.prevent="onConnectButtonTab"
            >
              Connect
            </VButton>
            <IconButton
              v-if="
                attributeTree.isBuildable &&
                !component.toDelete &&
                !parentHasExternalSources &&
                !attributeTree.attributeValue.externalSources?.length
              "
              ref="deleteButtonRef"
              v-tooltip="'Delete'"
              :tabIndex="
                attributeTree.isBuildable && !component.toDelete ? 0 : undefined
              "
              icon="trash"
              size="sm"
              iconTone="destructive"
              iconIdleTone="shade"
              loadingIcon="loader"
              :loading="bifrostingTrash"
              class="focus:outline focus:outline-action-500"
              @click="remove"
              @keydown.tab.stop.prevent="onDeleteButtonTab"
            />
          </div>
        </template>
        <ul v-if="!bifrostingTrash">
          <ComponentAttribute
            v-for="child in attributeTree.children"
            :key="child.id"
            :component="component"
            :attributeTree="child"
            :parentHasExternalSources="
              !!attributeTree.attributeValue.externalSources?.length
            "
            :forceReadOnly="
              props.forceReadOnly ||
              !!attributeTree.attributeValue.externalSources?.length
            "
            @save="
              (path, value, propKind, connectingComponentId) =>
                emit('save', path, value, propKind, connectingComponentId)
            "
            @delete="(path) => emit('delete', path)"
            @remove-subscription="(path) => emit('removeSubscription', path)"
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
              v-if="!attributeTree.attributeValue.externalSources?.length"
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
              + add "{{ displayName }}" item
            </VButton>
          </div>
        </div>
      </AttributeChildLayout>
    </template>
    <template v-else-if="!attributeTree.prop?.hidden">
      <AttributeInput
        :displayName="displayName"
        :path="attributeTree.attributeValue.path"
        :kind="attributeTree.prop?.widgetKind"
        :prop="attributeTree.prop"
        :validation="attributeTree.attributeValue.validation"
        :component="component"
        :value="attributeTree.attributeValue.value?.toString() ?? ''"
        :canDelete="
          attributeTree.isBuildable &&
          !component.toDelete &&
          !parentHasExternalSources
        "
        :externalSources="attributeTree.attributeValue.externalSources"
        :isArray="attributeTree.prop?.kind === 'array'"
        :isMap="attributeTree.prop?.kind === 'map'"
        :forceReadOnly="props.forceReadOnly || parentHasExternalSources"
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
import { useQuery } from "@tanstack/vue-query";
import {
  BifrostComponent,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import { PropKind } from "@/api/sdf/dal/prop";
import { AttributePath, ComponentId } from "@/api/sdf/dal/component";
import {
  getPossibleConnections,
  useMakeArgs,
  useMakeKey,
} from "@/store/realtime/heimdall";
import AttributeChildLayout from "./AttributeChildLayout.vue";
import AttributeInput from "./AttributeInput.vue";
import { AttrTree } from "../logic_composables/attribute_tree";
import { useApi, routes, componentTypes } from "../api_composables";
import { useWatchedForm } from "../logic_composables/watched_form";

const props = defineProps<{
  component: BifrostComponent;
  attributeTree: AttrTree;
  forceReadOnly?: boolean;
  parentHasExternalSources?: boolean;
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

const isBuildable = computed(
  () =>
    ["array", "map"].includes(props.attributeTree.prop?.kind ?? "") &&
    !props.component.toDelete,
);

const displayName = computed(() => {
  if (props.attributeTree.attributeValue.key)
    return props.attributeTree.attributeValue.key;
  else return props.attributeTree.prop?.name || "XXX";
});

const parentHasExternalSources = computed(() => {
  return props.parentHasExternalSources || false;
});

const addApi = useApi();

const emptyChildValue = () => {
  // Do I send `{}` for array of map/object or "" for array of string?
  // Answer by looking at my prop child
  const propTree = props.component.schemaVariant.propTree;
  const childProp =
    propTree.props[
      propTree.treeInfo[props.attributeTree.prop?.id ?? ""]?.children[0] ?? ""
    ];
  switch (childProp?.kind) {
    case "array":
      return [];
    case "map":
    case "object":
      return {};
    default:
      // string, number, boolean, etc.
      return "";
  }
};

const add = async (key?: string) => {
  if (props.component.toDelete) return;

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

  // Do I send `{}` for array of map/object or "" for array of string?
  // Answer by looking at my prop child
  const appendPath =
    `${props.attributeTree.attributeValue.path}/-` as AttributePath;
  const payload = {
    [appendPath]: emptyChildValue(),
  };
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
    const childPath =
      `${props.attributeTree.attributeValue.path}/${value.key}` as AttributePath;
    const payload = {
      [childPath]: emptyChildValue(),
    };
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
    emit("delete", props.attributeTree.attributeValue.path);
    bifrostingTrash.value = true;
  }
};

const removeSubscription = () => {
  if (props.attributeTree.attributeValue.path) {
    emit("removeSubscription", props.attributeTree.attributeValue.path);
  }
};

const showSubscriptionInput = ref(false);
const subscriptionInputRef = ref<InstanceType<typeof AttributeInput>>();

// Query for possible connections to look up component IDs
const makeArgs = useMakeArgs();
const makeKey = useMakeKey();
const queryKey = makeKey(EntityKind.PossibleConnections);

const possibleConnectionsQuery = useQuery({
  queryKey,
  queryFn: async () => {
    if (props.attributeTree.prop) {
      return await getPossibleConnections(
        makeArgs(EntityKind.PossibleConnections),
      );
    }
    return [];
  },
});

const createSubscription = (event: Event) => {
  // Prevent any event propagation that might close the input
  event.stopPropagation();
  event.preventDefault();

  showSubscriptionInput.value = true;

  // Use a longer delay to ensure the DOM is fully rendered and stable
  nextTick(() => {
    setTimeout(() => {
      subscriptionInputRef.value?.openInput();
    }, 50);
  });
};

const closeSubscriptionInput = () => {
  showSubscriptionInput.value = false;

  // Wait a bit for any state updates, then check for new subscriptions
  setTimeout(() => {
    if (props.attributeTree.attributeValue.externalSources?.length) {
      const externalSource =
        props.attributeTree.attributeValue.externalSources[0];
      if (externalSource) {
        // Look up the component ID from possible connections using the component name
        let connectingComponentId: ComponentId | undefined;

        if (possibleConnectionsQuery.data.value) {
          const matchingConnection = possibleConnectionsQuery.data.value.find(
            (conn) =>
              conn.componentName === externalSource.componentName &&
              conn.path === externalSource.path,
          );
          if (matchingConnection) {
            connectingComponentId =
              matchingConnection.componentId as ComponentId;
          }
        }

        emit(
          "save",
          props.attributeTree.attributeValue.path,
          externalSource.path,
          props.attributeTree.prop?.kind || PropKind.String,
          connectingComponentId,
        );
      }
    }
  }, 100);
};

const emit = defineEmits<{
  (
    e: "save",
    path: AttributePath,
    value: string,
    propKind: PropKind,
    connectingComponentId?: ComponentId,
  ): void;
  (e: "delete", path: AttributePath): void;
  (e: "removeSubscription", path: AttributePath): void;
}>();

const showingChildren = computed(
  () => hasChildren.value && props.attributeTree.children.length > 0,
);

const headerRef = ref<HTMLDivElement>();
const addButtonRef = ref<InstanceType<typeof VButton>>();
const connectButtonRef = ref<InstanceType<typeof VButton>>();
const deleteButtonRef = ref<InstanceType<typeof IconButton>>();

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
const onConnectButtonTab = (e: KeyboardEvent) => {
  handleTab(e, connectButtonRef.value?.$el);
};
const onDeleteButtonTab = (e: KeyboardEvent) => {
  handleTab(e, deleteButtonRef.value?.mainDivRef);
};
</script>
