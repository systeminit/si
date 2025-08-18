<!--  This whole component is very verbose and unoptimized, since it's made for the demo only  -->
<template>
  <ConfirmModal
    ref="modalRef"
    title="Initialize Workspace"
    confirmLabel="Initialize"
    size="2xl"
    @confirm="runOnboardRequest"
    @keydown.enter="runOnboardRequest"
  >
    <ErrorMessage v-if="requestError">{{ requestError }}</ErrorMessage>
    <div
      v-for="(field, idx) in formStructure"
      :key="idx"
      :class="
        clsx(
          'flex flex-row justify-between text-sm',
          field.ref === undefined && 'border-b border-neutral-400 pb-xs',
        )
      "
    >
      <template v-if="field.ref">
        <span class="mt-xs">
          {{ field.title }} {{ field.required ? "*" : "" }}
        </span>
        <div class="w-3/5 flex flex-col gap-2xs">
          <input
            v-model="field.ref.value"
            :type="field.type ?? 'text'"
            :class="
              clsx(
                'h-lg p-xs text-sm border font-mono cursor-text',
                'focus:outline-none focus:ring-0 focus:z-10',
                themeClasses(
                  'text-shade-100 bg-white border-neutral-400 focus:border-action-500',
                  'text-shade-0 bg-black border-neutral-600 focus:border-action-300',
                ),
              )
            "
            @paste="(ev: ClipboardEvent) => tryMatchOnPaste(ev, field)"
          />
          <span class="text-xs text-neutral-400">
            {{ field.hint }}
          </span>
        </div>
      </template>
      <template v-else>
        <span class="mt-xs font-bold text-md">
          {{ field.title }}
        </span>
        <div class="text-xs text-neutral-400 w-3/5">
          {{ field.hint }}
        </div>
      </template>
    </div>
  </ConfirmModal>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import {
  Modal,
  useModal,
  ErrorMessage,
  themeClasses,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import * as _ from "lodash-es";
import { componentTypes, routes, useApi } from "@/newhotness/api_composables";
import { encryptMessage } from "@/utils/messageEncryption";
import ConfirmModal from "@/newhotness/layout_components/ConfirmModal.vue";

const requestError = ref<string | undefined>();

// FORM FIELDS
const awsRegion = ref("us-east-1");
const credentialName = ref("My AWS Credential");
const credentialDescription = ref("");
const sessionToken = ref("");
const accessKeyId = ref("");
const secretAccessKey = ref("");
const assumeRole = ref("");
const endpoint = ref("");

const formStructure = [
  {
    title: "Region Data",
  },
  {
    title: "Region Name",
    ref: awsRegion,
    required: true,
  },
  {
    title: "Credential Metadata",
  },
  {
    title: "Credential Name",
    ref: credentialName,
    required: true,
  },
  {
    title: "Credential Description",
    ref: credentialDescription,
  },
  {
    title: "Credential Secret Data",
    hint: "If you paste a bash environment variable block on any of the following fields, we'll match and fill them in.",
  },
  {
    title: "Session Token",
    ref: sessionToken,
    type: "password",
  },
  {
    title: "Access Key ID",
    ref: accessKeyId,
    type: "password",
  },
  {
    title: "Secret Access Key",
    ref: secretAccessKey,
    type: "password",
  },
  {
    title: "Assume Role",
    ref: assumeRole,
    type: "password",
  },
  {
    title: "Endpoint",
    ref: endpoint,
    type: "password",
  },
];

type FormField = (typeof formStructure)[number];

const tryMatchOnPaste = (ev: ClipboardEvent, field: FormField) => {
  if (field.type !== "password") return;

  const text = ev.clipboardData?.getData("text/plain");

  if (!text) return;

  const valuesFromInput = _.compact(
    text.split("\n").map((e) => {
      const [_, key, value] = e.match(/export AWS_(.*)="(.*)"/) ?? [];
      if (!key || !value) return;

      return {
        key,
        value,
      };
    }),
  );

  if (!valuesFromInput.length) return;

  let matchedAValue = false;
  // Loop through form fields and try to match the values keys to the titles
  for (const { key, value } of valuesFromInput) {
    const matchedField = formStructure.find(
      (e) => e.title.replaceAll(" ", "_").toUpperCase() === key,
    );

    if (!matchedField || !matchedField.ref) continue;
    matchedAValue = true;
    matchedField.ref.value = value;
  }

  if (!matchedAValue) return;

  ev.preventDefault();
};

const initializeApi = useApi();
const keyApi = useApi();
const runOnboardRequest = async () => {
  // Encrypt secret
  const callApi = keyApi.endpoint<componentTypes.PublicKey>(
    routes.GetPublicKey,
    { id: "00000000000000000000000000" }, // TODO Remove component id from this endpoint's path, it's not needed
  );
  const resp = await callApi.get();
  const publicKey = resp.data;

  // Format cred values for encryption
  const credValue = (
    [
      ["SessionToken", sessionToken.value],
      ["AccessKeyId", accessKeyId.value],
      ["SecretAccessKey", secretAccessKey.value],
      ["AssumeRole", assumeRole.value],
      ["Endpoint", endpoint.value],
    ].filter(([_, value]) => value !== "") as [string, string][]
  ) // Remove empty values
    .reduce<{ [key: string]: string }>((acc, [key, value]) => {
      // make the pairs array into an object
      acc[key] = value;
      return acc;
    }, {});

  const crypted = await encryptMessage(credValue, publicKey);

  const call = initializeApi.endpoint(routes.ChangeSetInitializeAndApply);
  const { req } = await call.post({
    awsRegion: awsRegion.value,
    credential: {
      name: credentialName.value,
      description: credentialDescription.value,
      crypted,
      keyPairPk: publicKey.pk,
      version: componentTypes.SecretVersion.V1,
      algorithm: componentTypes.SecretAlgorithm.Sealedbox,
    },
  });

  if (initializeApi.ok(req)) {
    modalRef.value?.close();
  }
};

const modalRef = ref<InstanceType<typeof Modal>>();
const { open, close } = useModal(modalRef);
defineExpose({ open, close, isOpen: modalRef.value?.isOpen });
</script>
