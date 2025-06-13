<template>
  <div
    v-if="props.attributeTree.prop?.isOriginSecret && showSecretForm"
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
                'flex flex-col items-center gap-3xs font-sm [&>*]:w-full relative',
                index === Object.keys(secretFormData).length - 1
                  ? 'mb-xs'
                  : 'mb-[-1px]',
              )
            "
          >
            <secretForm.Field :name="fieldname">
              <template #default="{ field }">
                <div class="grid grid-cols-2">
                  <TruncateWithTooltip class="py-2xs">{{
                    fieldname
                  }}</TruncateWithTooltip>
                  <SecretInput
                    :field="field"
                    :fieldname="fieldname"
                    :placeholder="
                      attributeTree.secret ? getPlaceholder(fieldname) : ''
                    "
                  />
                </div>
                <div
                  v-for="error in field.state.meta.errors"
                  :key="error"
                  class="text-destructive-500 text-right pb-2xs"
                >
                  {{ error }}
                </div>
              </template>
            </secretForm.Field>
          </li>
          <!-- TODO(Wendy) - figure out tabbing for buttons -->
          <VButton
            :label="
              props.attributeTree.secret ? 'Replace Secret' : 'Add Secret'
            "
            :loading="wForm.bifrosting.value"
            loadingText="Saving Secret"
            tone="action"
            tabindex="-1"
            :disabled="!secretForm.state.canSubmit"
            @click="submitSecretForm"
          />
        </ul>
      </div>
    </AttributeChildLayout>
  </div>
  <!-- TODO(nick): add the ability to remove a subscription -->
  <AttributeInput
    v-else
    :displayName="props.attributeTree.prop?.name ?? 'Secret Value'"
    :attributeValueId="props.attributeTree.attributeValue.id"
    :path="props.attributeTree.attributeValue.path ?? ''"
    :kind="props.attributeTree.prop?.widgetKind"
    :prop="props.attributeTree.prop"
    :component="component"
    :externalSources="props.attributeTree.attributeValue.externalSources"
    :value="props.attributeTree.secret?.name?.toString() ?? ''"
    :canDelete="false"
    :disableInputWindow="props.attributeTree.prop?.isOriginSecret"
    isSecret
    @selected="openSecretForm"
    @save="
      (path, id, value, _kind, connectingComponentId) =>
        save(path, id, value, connectingComponentId)
    "
    @remove-subscription="removeSubscription"
  />
</template>

<script setup lang="ts">
import { computed, nextTick, ref } from "vue";
import {
  themeClasses,
  TruncateWithTooltip,
  VButton,
} from "@si/vue-lib/design-system";
import { useRoute } from "vue-router";
import clsx from "clsx";
import { BifrostComponent } from "@/workers/types/entity_kind_types";
import { encryptMessage } from "@/utils/messageEncryption";
import AttributeChildLayout from "./AttributeChildLayout.vue";
import AttributeInput from "./AttributeInput.vue";
import { AttrTree } from "../AttributePanel.vue";
import { useApi, routes, componentTypes } from "../api_composables";
import { useWatchedForm } from "../logic_composables/watched_form";
import SecretInput from "./SecretInput.vue";

const props = defineProps<{
  component: BifrostComponent;
  attributeTree: AttrTree;
}>();

const displayName = computed(() => {
  if (props.attributeTree.attributeValue.key)
    return props.attributeTree.attributeValue.key;
  else return props.attributeTree.prop?.name || "XXX";
});

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

const removeSubscription = async (path: string, _id: string) => {
  const call = api.endpoint<{ success: boolean }>(
    routes.UpdateComponentAttributes,
    { id: props.component.id },
  );

  const payload: componentTypes.UpdateComponentAttributesArgs = {};
  path = path.replace("root", ""); // endpoint doesn't want it

  payload[path] = {
    $source: null,
  };

  await call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
};

const route = useRoute();

const secretApi = useApi();
const keyApi = useApi();

const wForm = useWatchedForm<Record<string, string>>(
  `component.av.secret.${props.attributeTree.prop?.id}`,
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
      return {
        fields: {
          Name: !value.Name ? "Name required" : undefined,
        },
      };
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

const secretFormRef = ref<HTMLDivElement>();

const onClick = (e: MouseEvent) => {
  const target = e.target;
  if (!(target instanceof Element)) {
    return;
  }
  const el = secretFormRef.value;
  if (el && !el.contains(target)) {
    secretFormOpen.value = false;
    removeListeners();
  }
};

const addListeners = () => {
  window.addEventListener("mousedown", onClick);
};
const removeListeners = () => {
  window.removeEventListener("mousedown", onClick);
};

const submitSecretForm = async () => {
  await secretForm.handleSubmit();
  secretFormOpen.value = false;
};
</script>
