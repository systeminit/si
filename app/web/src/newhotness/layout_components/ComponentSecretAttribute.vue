<template>
  <AttributeChildLayout>
    <template #header>
      <div class="flex flex-row items-center gap-2xs">
        <div>{{ displayName }}</div>
      </div>
    </template>
    <template v-if="props.attributeTree.prop?.isOriginSecret">
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
            :label="
              props.attributeTree.secret ? 'Replace Secret' : 'Add Secret'
            "
            :loading="wForm.bifrosting.value"
            loadingText="Saving Secret"
            tone="action"
            :disabled="!secretForm.state.canSubmit"
            @click="() => secretForm.handleSubmit()"
          />
        </ul>
      </div>
    </template>
    <template v-else>
      <AttributeInput
        :displayName="props.attributeTree.prop?.name ?? 'Secret Value'"
        :attributeValueId="props.attributeTree.attributeValue.id"
        :path="props.attributeTree.attributeValue.path ?? ''"
        :kind="props.attributeTree.prop?.widgetKind"
        :prop="props.attributeTree.prop"
        :externalSources="props.attributeTree.attributeValue.externalSources"
        :value="props.attributeTree.secret?.name?.toString() ?? ''"
        :canDelete="false"
        isSecret
        @save="
          (path, id, value, connectingComponentId) =>
            save(path, id, value, connectingComponentId)
        "
      />
    </template>
  </AttributeChildLayout>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { VButton } from "@si/vue-lib/design-system";
import { useRoute } from "vue-router";
import clsx from "clsx";
import { BifrostComponent } from "@/workers/types/entity_kind_types";
import { encryptMessage } from "@/utils/messageEncryption";
import AttributeChildLayout from "./AttributeChildLayout.vue";
import AttributeInput from "./AttributeInput.vue";
import { AttrTree } from "../AttributePanel.vue";
import { useApi, routes, componentTypes } from "../api_composables";
import { useWatchedForm } from "../logic_composables/watched_form";

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
    return { Name: "", Description: "", ...form };
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
</script>
