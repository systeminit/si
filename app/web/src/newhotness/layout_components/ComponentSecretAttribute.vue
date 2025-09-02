<template>
  <div
    v-if="attributeTree.prop?.isOriginSecret && showSecretForm"
    ref="secretFormRef"
  >
    <AttributeChildLayout>
      <template #header>
        <div class="flex flex-row items-center gap-2xs">
          <div>{{ displayName }}</div>
        </div>
      </template>
      <div
        :class="
          clsx(
            'p-xs flex flex-col gap-xs',
            themeClasses('bg-shade-0', 'bg-neutral-900'),
          )
        "
      >
        <div
          :class="
            clsx(
              'text-sm italic',
              themeClasses('text-neutral-600', 'text-neutral-400'),
            )
          "
        >
          Secret data entered will be encrypted. Secret data can always be
          replaced, but only the name and description can be viewed.
        </div>
        <ul class="flex flex-col">
          <li
            v-for="(fieldname, index) in Object.keys(secretFormData)"
            :key="fieldname"
            :class="
              clsx(
                'flex flex-col items-center gap-3xs text-sm [&>*]:w-full relative',
                index === Object.keys(secretFormData).length - 1
                  ? 'mb-xs'
                  : 'mb-[-1px]',
              )
            "
          >
            <secretForm.Field :name="fieldname">
              <template #default="{ field }">
                <div class="grid grid-cols-2">
                  <div class="py-2xs">
                    <AttributeInputRequiredProperty
                      :text="fieldname"
                      :showAsterisk="isFieldRequired(fieldname)"
                    />
                  </div>
                  <SecretInput
                    :field="field"
                    :fieldname="fieldname"
                    :placeholder="
                      attributeTree.secret ? getPlaceholder(fieldname) : ''
                    "
                  />
                </div>
                <!-- Validation errors are intentionally not displayed -->
              </template>
            </secretForm.Field>
          </li>
          <!-- TODO(Wendy) - figure out tabbing for buttons -->
          <VButton
            :label="attributeTree.secret ? 'Replace Secret' : 'Add Secret'"
            :loading="wForm.bifrosting.value"
            loadingText="Saving Secret"
            tone="action"
            tabindex="-1"
            :disabled="!secretForm.state.canSubmit"
            @click="submitSecretForm"
          />
        </ul>
        <div
          v-if="featureFlagsStore.DEFAULT_SUBS"
          :class="
            clsx(
              'border w-full',
              themeClasses('border-neutral-300', 'border-neutral-600'),
            )
          "
        >
          <input
            :id="`default-subs-checkbox-${attributeTree.prop?.id}`"
            type="checkbox"
            :checked="attributeTree.attributeValue.isDefaultSource"
            @input="
              (ev) =>
                toggleIsDefaultSource(ev, attributeTree.attributeValue.path)
            "
          />
          <label :for="`default-subs-checkbox-${attributeTree.prop?.id}`">
            Make this the default subscription for new components
          </label>
        </div>
      </div>
    </AttributeChildLayout>
  </div>
  <!-- TODO(nick): add the ability to remove a subscription -->
  <AttributeInput
    v-else
    :displayName="attributeTree.prop?.name ?? 'Secret Value'"
    :attributeValueId="attributeTree.attributeValue.id"
    :path="attributeTree.attributeValue.path ?? ''"
    :kind="attributeTree.prop?.widgetKind"
    :prop="attributeTree.prop"
    :validation="attributeTree.attributeValue.validation"
    :component="component"
    :externalSources="attributeTree.attributeValue.externalSources"
    :value="attributeTree.secret?.name?.toString() ?? ''"
    :canDelete="false"
    :disableInputWindow="attributeTree.prop?.isOriginSecret"
    isSecret
    @selected="openSecretForm"
    @save="
      (path, value, _kind, connectingComponentId) =>
        save(path, value, connectingComponentId)
    "
    @remove-subscription="removeSubscription"
  />
</template>

<script setup lang="ts">
import { computed, nextTick, ref } from "vue";
import { themeClasses, VButton } from "@si/vue-lib/design-system";
import { useRoute } from "vue-router";
import clsx from "clsx";
import {
  BifrostComponent,
  ComponentInList,
} from "@/workers/types/entity_kind_types";
import { encryptMessage } from "@/utils/messageEncryption";
import { AttributePath, ComponentId } from "@/api/sdf/dal/component";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import AttributeChildLayout from "./AttributeChildLayout.vue";
import AttributeInput from "./AttributeInput.vue";
import AttributeInputRequiredProperty from "./AttributeInputRequiredProperty.vue";
import { AttrTree } from "../logic_composables/attribute_tree";
import { useApi, routes, componentTypes } from "../api_composables";
import { useWatchedForm } from "../logic_composables/watched_form";
import SecretInput from "./SecretInput.vue";
import { MouseDetails, mouseEmitter } from "../logic_composables/emitters";

const featureFlagsStore = useFeatureFlagsStore();

const props = defineProps<{
  component: BifrostComponent | ComponentInList;
  attributeTree: AttrTree;
}>();

const emit = defineEmits<{
  (
    e: "setDefaultSubscriptionSource",
    path: AttributePath,
    setTo: boolean,
  ): void;
}>();

const displayName = computed(() => {
  if (props.attributeTree.attributeValue.key)
    return props.attributeTree.attributeValue.key;
  else return props.attributeTree.prop?.name || "XXX";
});

const isFieldRequired = (fieldname: string): boolean => {
  // Only "Name" is required for secrets (no other secrets have validations)
  return fieldname === "Name";
};

const secretFormData = computed(() => {
  if (
    props.attributeTree.prop?.isOriginSecret &&
    props.attributeTree.prop?.secretDefinition
  ) {
    const form = props.attributeTree.prop.secretDefinition.formData
      .flatMap((row) => row.name)
      .reduce((obj, name) => {
        obj[name] = "";
        return obj;
      }, {} as Record<string, string>);
    return {
      Name: props.attributeTree.secret?.name ?? "",
      Description: props.attributeTree.secret?.description ?? "",
      ...form,
    };
  } else return {};
});

const saveApi = useApi();
const save = async (
  path: AttributePath,
  value: string,
  connectingComponentId?: ComponentId,
) => {
  const call = saveApi.endpoint<{ success: boolean }>(
    routes.UpdateComponentAttributes,
    { id: props.component.id },
  );
  const payload: componentTypes.UpdateComponentAttributesArgs = {};
  payload[path] = value;
  if (connectingComponentId) {
    payload[path] = {
      $source: { component: connectingComponentId, path: value },
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

const removeSubscriptionApi = useApi();
const removeSubscription = async (path: AttributePath) => {
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
};

const route = useRoute();

const secretApi = useApi();
const keyApi = useApi();

const wForm = useWatchedForm<Record<string, string>>(
  `component.av.secret.${props.attributeTree.prop?.id}`,
  false,
  true,
);
const secretForm = wForm.newForm({
  data: secretFormData,
  onSubmit: async ({ value }) => {
    const definition = props.attributeTree.prop?.secretDefinition?.label;
    const propId = props.attributeTree.prop?.id;
    if (!definition) throw new Error("Secret Definition Required");
    if (!propId) throw new Error("Secret Definition Prop Id required");

    const callApi = keyApi.endpoint<componentTypes.PublicKey>(
      routes.GetPublicKey,
      { id: props.component.id },
    );
    const resp = await callApi.get();
    const publicKey = resp.data;

    const filteredValue = Object.fromEntries(
      Object.entries(value).filter(([_key, val]) => val !== ""),
    );

    const name = filteredValue.Name ?? "";
    delete filteredValue.Name;

    const description = filteredValue.Description ?? "";
    delete filteredValue.Description;

    const crypted = await encryptMessage(filteredValue, publicKey);

    const payload: componentTypes.CreateSecret = {
      name,
      attributeValueId: props.attributeTree.attributeValue.id,
      propId,
      definition,
      description,
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
      secretApi.navigateToNewChangeSet(
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
  validators: {
    onSubmit: ({ value }) => {
      if (!value.Name || value.Name.trim() === "") {
        return " "; // Return non-empty string to indicate validation failure
      }
      return undefined; // Return undefined for successful validation
    },
  },
  watchFn: () => props.attributeTree.secret,
});

const getPlaceholder = (fieldname: string) => {
  if (!props.attributeTree.secret) return "";

  if (fieldname === "Name") {
    return props.attributeTree.secret.name;
  } else if (fieldname === "Description") {
    return props.attributeTree.secret.description;
  } else return "empty";
};

const secretFormOpen = ref(false);
const showSecretForm = computed(
  () => !props.attributeTree.secret || secretFormOpen.value,
);
const openSecretForm = () => {
  secretFormOpen.value = true;
  addListeners();
  nextTick(() => {
    const inputs = secretFormRef.value?.getElementsByTagName("input");

    if (inputs && inputs[0]) {
      inputs[0].focus();
    }
  });
};
const closeSecretForm = () => {
  secretFormOpen.value = false;
  removeListeners();
};

const secretFormRef = ref<HTMLDivElement>();

const onMouseDown = (e: MouseDetails["mousedown"]) => {
  const target = e.target;
  if (!(target instanceof Element)) {
    return;
  }
  const el = secretFormRef.value;
  if (el && !el.contains(target)) {
    closeSecretForm();
  }
};

const addListeners = () => {
  mouseEmitter.on("mousedown", onMouseDown);
};
const removeListeners = () => {
  mouseEmitter.off("mousedown", onMouseDown);
};

const submitSecretForm = async () => {
  await secretForm.handleSubmit();
  closeSecretForm();
};

const toggleIsDefaultSource = (event: Event, path: AttributePath) => {
  const checked = (event.target as HTMLInputElement).checked;
  emit("setDefaultSubscriptionSource", path, checked);
};
</script>
