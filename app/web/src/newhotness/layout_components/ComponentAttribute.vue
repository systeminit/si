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
    :isDefaultSource="false"
    :forceReadOnly="false"
    :hasSocketConnection="hasSocketConnections"
    @close="closeSubscriptionInput"
    @save="(...args) => emit('save', ...args)"
  />
  <li
    v-else-if="showingChildren || !attributeTree.prop?.hidden"
    :class="clsx('flex flex-col', !showingChildren && 'mb-[-1px]')"
  >
    <template v-if="showingChildren">
      <AttributeChildLayout
        :sticky="
          attributeTree.prop?.kind === 'array' ||
          attributeTree.prop?.kind === 'map' ||
          (attributeTree.prop?.kind === 'object' && isFirstChild)
        "
        :stickyTopOffset="(stickyDepth || 0) * 36"
        :stickyZIndex="10 - (stickyDepth || 0)"
      >
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
            <div
              :class="
                attributeTree.prop?.kind === 'array' ||
                attributeTree.prop?.kind === 'map' ||
                (stickyDepth && stickyDepth > 0)
                  ? 'text-sm'
                  : ''
              "
            >
              {{ displayName }}
            </div>
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
        <ul v-if="!bifrostingTrash" class="list-none">
          <ComponentAttribute
            v-for="(child, index) in attributeTree.children"
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
            :stickyDepth="(stickyDepth || 0) + 1"
            :isFirstChild="index === 0"
            @save="
              (path, value, propKind, connectingComponentId) =>
                emit('save', path, value, propKind, connectingComponentId)
            "
            @delete="(path) => emit('delete', path)"
            @remove-subscription="(path) => emit('removeSubscription', path)"
            @set-default-subscription-source="
              (path, setTo) => emit('setDefaultSubscriptionSource', path, setTo)
            "
            @add="(...args) => emit('add', ...args)"
            @set-key="(...args) => emit('setKey', ...args)"
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
              v-if="
                !attributeTree.attributeValue.externalSources?.length &&
                props.attributeTree.prop?.kind !== 'map'
              "
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
              :loading="addButtonBifrosting"
              :disabled="addButtonBifrosting"
              loadingIcon="loader"
              :tabindex="0"
              @click="add"
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
        :hasSocketConnection="hasSocketConnections"
        :isDefaultSource="attributeTree.attributeValue.isDefaultSource"
        @save="(...args) => emit('save', ...args)"
        @delete="(...args) => emit('delete', ...args)"
        @set-default-subscription-source="
          (path, setTo) => emit('setDefaultSubscriptionSource', path, setTo)
        "
        @remove-subscription="(...args) => emit('removeSubscription', ...args)"
        @add="(...args) => add(...args)"
      />
    </template>
  </li>
</template>

<script setup lang="ts">
import { computed, nextTick, ref } from "vue";
import { VButton, IconButton, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useQuery } from "@tanstack/vue-query";
import {
  BifrostComponent,
  ComponentInList,
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
import { useApi, UseApi } from "../api_composables";
import { useWatchedForm } from "../logic_composables/watched_form";

const props = defineProps<{
  component: BifrostComponent | ComponentInList;
  attributeTree: AttrTree;
  forceReadOnly?: boolean;
  parentHasExternalSources?: boolean;
  stickyDepth?: number;
  isFirstChild?: boolean;
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

const hasSocketConnections = computed(() => {
  if (!props.attributeTree || !props.attributeTree.attributeValue) return false;
  return props.attributeTree.attributeValue.hasSocketConnection;
});

const displayName = computed(() => {
  if (props.attributeTree.attributeValue.key)
    return props.attributeTree.attributeValue.key;
  else return props.attributeTree.prop?.name || "XXX";
});

const parentHasExternalSources = computed(() => {
  return props.parentHasExternalSources || false;
});

const addButtonBifrosting = computed(() => {
  return props.attributeTree.prop?.kind === "map"
    ? wForm.bifrosting.value
    : false;
});

export type NewChildValue = [] | object | "";
const emptyChildValue = (): NewChildValue => {
  // Do I send `{}` for array of map/object or "" for array of string?
  // Answer by looking at my prop child

  // no answer to this question without a prop
  if (!props.attributeTree.prop) return "";

  switch (props.attributeTree.prop.childKind) {
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

const wForm = useWatchedForm<{ key: string }>(
  `component.av.key.${props.attributeTree.prop?.id}`,
);
const keyData = ref({ key: "" });
const keyForm = wForm.newForm({
  data: keyData,
  onSubmit: async ({ value }) => {
    emit("setKey", props.attributeTree, value.key, emptyChildValue());
  },
  watchFn: () => props.attributeTree.children.length,
});

// map (all except the first, see below) and object keys come through this FN
const saveKeyIfFormValid = async () => {
  if (keyForm.fieldInfo.key.instance?.state.meta.isDirty) {
    if (!keyForm.baseStore.state.isSubmitted) {
      await saveKey();
    }
  }
};

const saveKey = async () => {
  await keyForm.handleSubmit();
  // The keyForm has a watchFn that automatically handles bifrosting reset
  // when props.attributeTree.children.length changes
};

const addApi = useApi();
const add = (keyName?: string) => {
  // when adding a map key (for the first time), you're doing it from the child, which gives you the key name
  if (props.attributeTree.prop?.kind === "map") {
    if (!keyName) throw new Error("Expected key name");
    emit("setKey", props.attributeTree, keyName, emptyChildValue());
    return;
  }

  // when adding a net-new array, you're doing it from within this component (keyName does not apply)
  emit("add", addApi, props.attributeTree, emptyChildValue());
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
  (
    e: "setDefaultSubscriptionSource",
    path: AttributePath,
    setTo: boolean,
  ): void;
  (e: "removeSubscription", path: AttributePath): void;
  (e: "add", api: UseApi, attributeTree: AttrTree, value: NewChildValue): void;
  (
    e: "setKey",
    attributeTree: AttrTree,
    key: string,
    value: NewChildValue,
  ): void;
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
