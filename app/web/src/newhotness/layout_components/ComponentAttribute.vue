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
    @selected="focused"
    @close="closeSubscriptionInput"
    @save="(...args) => emit('save', ...args)"
    @handleTab="handleTab"
  />
  <li
    v-else-if="!attributeTree.prop?.hidden"
    :class="clsx('flex flex-col', !showingChildren && 'mb-[-1px]')"
  >
    <template v-if="showingChildren">
      <AttributeChildLayout
        :sticky="
          attributeTree.prop?.kind === 'array' ||
          attributeTree.prop?.kind === 'map' ||
          attributeTree.prop?.kind === 'object'
        "
        :stickyTopOffset="(stickyDepth || 0) * 36"
        :stickyZIndex="10 - (stickyDepth || 0)"
      >
        <template #header>
          <div
            ref="headerRef"
            :class="
              clsx(
                'flex flex-row items-center gap-2xs flex-1 min-w-0 justify-between',
                attributeTree.isBuildable &&
                  'focus:outline-none group/attributeheader',
              )
            "
            @keydown.tab.stop.prevent="onHeaderTab"
            @keydown.enter.stop.prevent="remove"
            @keydown.delete.stop.prevent="remove"
          >
            <!-- displayName aligned left -->
            <TruncateWithTooltip
              :class="
                clsx(
                  'flex-1 min-w-0 max-w-fit',
                  attributeTree.prop?.kind === 'array' ||
                    attributeTree.prop?.kind === 'map' ||
                    (stickyDepth && stickyDepth > 0)
                    ? 'text-sm'
                    : '',
                )
              "
            >
              {{ displayName }}
            </TruncateWithTooltip>

            <!-- everything else aligned right -->
            <div
              class="flex flex-row flex-1 min-w-0 max-w-fit gap-2xs items-center"
            >
              <div
                v-if="attributeTree.attributeValue.externalSources?.length"
                class="flex flex-row items-center gap-2xs text-xs flex-1 min-w-0"
              >
                <TruncateWithTooltip class="flex-1 min-w-0 max-w-fit">
                  <span
                    :class="
                      themeClasses('text-neutral-500', 'text-neutral-400')
                    "
                  >
                    Set via subscription to
                  </span>
                  <span
                    :class="
                      themeClasses(
                        'text-newhotness-purplelight',
                        'text-newhotness-purpledark',
                      )
                    "
                  >
                    {{
                      attributeTree.attributeValue.externalSources[0]
                        ?.componentName
                    }}
                  </span>
                  <span
                    :class="
                      themeClasses('text-neutral-600', 'text-neutral-400')
                    "
                  >
                    {{ attributeTree.attributeValue.externalSources[0]?.path }}
                  </span>
                </TruncateWithTooltip>
                <NewButton
                  tooltip="Remove subscription"
                  tooltipPlacement="top"
                  icon="x"
                  tone="empty"
                  :class="
                    clsx(
                      'active:bg-white active:text-black',
                      themeClasses(
                        'hover:bg-neutral-200',
                        'hover:bg-neutral-600',
                      ),
                    )
                  "
                  @click="removeSubscription"
                />
              </div>
              <NewButton
                v-if="
                  attributeTree.isBuildable &&
                  !component.toDelete &&
                  !parentHasExternalSources &&
                  !props.forceReadOnly &&
                  attributeTree.prop?.kind === 'object' &&
                  !attributeTree.attributeValue.externalSources?.length
                "
                ref="connectButtonRef"
                tooltip="Create subscription"
                :tabIndex="
                  attributeTree.isBuildable && !component.toDelete
                    ? 0
                    : undefined
                "
                label="Connect"
                :class="
                  clsx(
                    'focus:outline flex-none',
                    themeClasses(
                      'focus:outline-action-500',
                      'focus:outline-action-300',
                    ),
                  )
                "
                @keydown.enter.stop.prevent="createSubscription"
                @click.stop.prevent="createSubscription"
                @keydown.tab.stop.prevent="onConnectButtonTab"
              />
              <NewButton
                v-if="
                  attributeTree.isBuildable &&
                  !component.toDelete &&
                  !parentHasExternalSources &&
                  !props.forceReadOnly &&
                  !attributeTree.attributeValue.externalSources?.length
                "
                ref="deleteButtonRef"
                tooltip="Delete"
                tooltipPlacement="top"
                :tabIndex="
                  attributeTree.isBuildable && !component.toDelete
                    ? 0
                    : undefined
                "
                icon="trash"
                tone="destructive"
                loadingIcon="loader"
                loadingText=""
                :loading="bifrostingTrash"
                :class="
                  clsx(
                    'focus:outline flex-none',
                    themeClasses(
                      'focus:outline-action-500',
                      'focus:outline-action-300',
                    ),
                  )
                "
                @click="remove"
                @keydown.enter.stop.prevent="remove"
                @keydown.tab.stop.prevent="onDeleteButtonTab"
              />
            </div>
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
            @focused="(path) => focused(path)"
          />
        </ul>
        <div
          v-if="isBuildable"
          class="grid grid-cols-2 items-center gap-2xs relative"
        >
          <template
            v-if="attributeTree.prop?.kind === 'map' && !props.forceReadOnly"
          >
            <keyForm.Field
              name="key"
              :validators="{
                onChange: keyValidator,
                onBlur: keyValidator,
              }"
            >
              <template #default="{ field }">
                <input
                  :class="
                    clsx(
                      'block ml-auto border w-full h-lg font-mono text-sm order-2',
                      'focus:outline-none focus:shadow-none focus:ring-0',
                      field.state.meta.errors.length > 0
                        ? themeClasses(
                            'text-black bg-white !border-destructive-600 disabled:bg-neutral-200',
                            'text-white bg-black !border-destructive-400 disabled:bg-neutral-900',
                          )
                        : themeClasses(
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
                  @keydown.enter.stop.prevent="saveKeyIfFormValid"
                />
                <template v-if="field.state.meta.errors.length > 0">
                  <div class="order-3" />
                  <div
                    :class="
                      clsx(
                        'text-sm mb-xs order-4',
                        themeClasses(
                          'text-destructive-600',
                          'text-destructive-200',
                        ),
                      )
                    "
                  >
                    {{ field.state.meta.errors[0] }}
                  </div>
                </template>
              </template>
            </keyForm.Field>
          </template>
          <div class="p-xs">
            <NewButton
              v-if="
                !attributeTree.attributeValue.externalSources?.length &&
                !props.forceReadOnly
              "
              ref="addButtonRef"
              :loading="addButtonBifrosting"
              :disabled="addButtonBifrosting"
              loadingIcon="loader"
              :tabIndex="0"
              @click="onAddButtonClick"
              @keydown.tab.stop.prevent="onAddButtonTab"
            >
              + add "{{ displayName }}" item
            </NewButton>
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
          !parentHasExternalSources &&
          !props.forceReadOnly
        "
        :externalSources="attributeTree.attributeValue.externalSources"
        :isArray="attributeTree.prop?.kind === 'array'"
        :isMap="attributeTree.prop?.kind === 'map'"
        :forceReadOnly="props.forceReadOnly || parentHasExternalSources"
        :hasSocketConnection="hasSocketConnections"
        :isDefaultSource="attributeTree.attributeValue.isDefaultSource"
        @selected="focused"
        @save="(...args) => emit('save', ...args)"
        @delete="(...args) => emit('delete', ...args)"
        @set-default-subscription-source="
          (path, setTo) => emit('setDefaultSubscriptionSource', path, setTo)
        "
        @remove-subscription="(...args) => emit('removeSubscription', ...args)"
        @add="(...args) => add(...args)"
        @handleTab="handleTab"
      />
    </template>
  </li>
</template>

<script setup lang="ts">
import { computed, nextTick, ref } from "vue";
import {
  themeClasses,
  NewButton,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
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
import { handleTab } from "../logic_composables/controls";

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
const keyData = computed<{ key: string }>(() => {
  return { key: "" };
});
const keyForm = wForm.newForm({
  data: keyData,
  onSubmit: async () => {
    // DO NOT SUBMIT THIS FORM, instead use saveKeyIfFormValid
  },
  validators: {
    onSubmit: ({ value }) => {
      if (value.key.trim().length === 0) {
        return "Key name is required";
      } else if (existingKeys.value.includes(keyForm.state.values.key)) {
        return "That key name is already in use";
      }
      return undefined;
    },
  },
  watchFn: () => {
    return props.attributeTree.children.length;
  },
});
const keyValidator = ({ value }: { value: string }) => {
  if (value.trim().length === 0) {
    return "Key name is required";
  } else if (existingKeys.value.includes(value)) {
    return "That key name is already in use";
  }
  return undefined;
};

const existingKeys = computed(() => {
  if (props.attributeTree.prop?.kind === "map") {
    return props.attributeTree.children.map(
      (child) => child.attributeValue.key as string,
    );
  } else {
    return [];
  }
});

// map (all except the first, see below) and object keys come through this FN
const saveKeyIfFormValid = async () => {
  if (
    keyForm.fieldInfo.key.instance?.state.meta.isDirty &&
    !keyForm.baseStore.state.isSubmitted
  ) {
    add(keyForm.state.values.key);
  } else {
    keyForm.validateAllFields("blur");
  }
};

const addApi = useApi();
const add = (keyName?: string) => {
  // when adding a map key (for the first time), you're doing it from the child, which gives you the key name
  if (props.attributeTree.prop?.kind === "map") {
    if (
      !keyName ||
      keyName.trim().length === 0 ||
      existingKeys.value.includes(keyName)
    ) {
      keyForm.validateAllFields("blur");
      return;
    }
    emit("setKey", props.attributeTree, keyName, emptyChildValue());
    wForm.reset(keyForm);
    return;
  }

  // when adding a net-new array, you're doing it from within this component (keyName does not apply)
  emit("add", addApi, props.attributeTree, emptyChildValue());
};
const onAddButtonClick = () => {
  if (props.attributeTree.prop?.kind === "map") {
    add(keyForm.state.values.key);
  } else {
    add();
  }
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

const focused = (path?: string) => {
  if (!path) path = props.attributeTree.attributeValue.path;
  emit("focused", path);
};

const createSubscription = (event: Event) => {
  // Prevent any event propagation that might close the input
  event.stopPropagation();
  event.preventDefault();

  showSubscriptionInput.value = true;

  // Use a longer delay to ensure the DOM is fully rendered and stable
  nextTick(() => {
    setTimeout(() => {
      subscriptionInputRef.value?.openInput();
      focused();
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
  (e: "focused", path: string): void;
}>();

const showingChildren = computed(
  () => hasChildren.value && props.attributeTree.children.length > 0,
);

const headerRef = ref<HTMLDivElement>();
const addButtonRef = ref<InstanceType<typeof NewButton>>();
const connectButtonRef = ref<InstanceType<typeof NewButton>>();
const deleteButtonRef = ref<InstanceType<typeof NewButton>>();

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
  handleTab(e, deleteButtonRef.value?.mainElRef);
};
</script>
