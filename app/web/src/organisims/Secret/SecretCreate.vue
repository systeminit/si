<template>
  <div class="flex flex-col w-full">
    <div class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-sm text-right text-gray-400 align-middle">
        <label for="secret-name-textbox">Secret Name:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          id="secret-name-textbox"
          v-model="secretName"
          size="xs"
          name="secretName"
          placeholder="secret name"
          :is-show-type="false"
          required
        />
      </div>
    </div>
    <div class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-sm text-right text-gray-400 align-middle">
        <label for="secret-password-textbox">Secret Kind:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiSelect
          id="secret-password-textbox"
          v-model="secretKind"
          size="xs"
          name="secretKind"
          :options="secretKindOptionList"
          required
        />
      </div>
    </div>

    <SecretCreateFields
      v-if="secretKindFields"
      v-model="secretMessage"
      :secret-kind-fields="secretKindFields"
    />

    <div class="flex justify-end w-full">
      <div class="pr-2">
        <SiButton
          size="xs"
          label="Cancel"
          kind="cancel"
          :icon="null"
          @click="cancel"
        />
      </div>
      <div>
        <SiButton
          size="xs"
          label="Create"
          kind="save"
          :icon="null"
          :disabled="!enableCreateButton"
          @click="create"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Base64 } from "js-base64";
import _ from "lodash";
// NOTE(nick): we have our own "tweetnacl-sealedbox-js" with types for TS.
// @ts-ignore
import sealedBox from "tweetnacl-sealedbox-js";
import { from } from "rxjs";
import { switchMap } from "rxjs/operators";
import { computed, ref, watch } from "vue";
import { refFrom } from "vuse-rx";
import {
  SecretAlgorithm,
  SecretKind,
  SecretKindFields,
  SecretObjectType,
  SecretVersion,
} from "@/api/sdf/dal/secret";
import SiButton from "@/atoms/SiButton.vue";
import SiSelect, { SelectPropsOption } from "@/atoms/SiSelect.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import SecretCreateFields from "@/organisims/Secret/SecretCreateFields.vue";
import { ChangeSetService } from "@/service/change_set";
import { GlobalErrorService } from "@/service/global_error";
import { SecretService } from "@/service/secret";

const emit = defineEmits(["cancel", "submit"]);

const editMode = refFrom<boolean | undefined>(
  ChangeSetService.currentEditMode(),
);

const secretName = ref<string>("");

const secretKind = ref<SecretKind | null>(null);

const secretMessage = ref<Record<string, string>>({});

const secretKindFields = computed((): SecretKindFields | undefined => {
  if (!secretKind.value || !allSecretKindFields.value) {
    return undefined;
  }

  // Find the entry for the selected `SecretKind`--if not found, then
  // `undefined` is returned
  const secretKindFields = _.find(
    allSecretKindFields.value,
    (secretKindFields) => secretKindFields.secretKind == secretKind.value,
  );
  return secretKindFields;
});

const enableCreateButton = computed((): boolean => {
  // Early return if any form fields have missing or empty values in the
  // message
  if (!secretKindFields.value) {
    return false;
  } else {
    for (const field of secretKindFields.value.fields) {
      if (!secretMessage.value[field.keyName]) {
        // Key is not yet set for this field in the message
        return false;
      } else if (secretMessage.value[field.keyName].length == 0) {
        // The value for this field is empty
        return false;
      }
    }
  }

  const inEditMode = editMode.value != undefined && editMode.value;
  const secretKindSelected = !!secretKind.value;
  const namePopulated = secretName.value.length > 0;

  return inEditMode && namePopulated && secretKindSelected;
});

const allSecretKindFields = refFrom<SecretKindFields[] | undefined>(
  SecretService.listSecretKindFields().pipe(
    switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return from([undefined]);
      } else {
        return from([response.fields]);
      }
    }),
  ),
);

const secretKindOptionList = computed((): SelectPropsOption[] => {
  let options: SelectPropsOption[] = [{ label: "", value: "" }];
  if (allSecretKindFields.value) {
    for (const kindFields of allSecretKindFields.value) {
      options.push({
        label: kindFields.displayName,
        value: String(kindFields.secretKind),
      });
    }
  }
  return options;
});

const clear = () => {
  secretName.value = "";
  secretKind.value = null;
  for (const key of Object.keys(secretMessage.value)) {
    delete secretMessage.value[key];
  }
};

const cancel = () => {
  clear();
  emit("cancel");
};

const create = async (): Promise<void> => {
  const kind = secretKind.value;
  if (!kind) {
    return;
  }
  const publicKey = await getPublicKey();
  if (!publicKey) {
    return;
  }
  const crypted = encryptMessage(secretMessage.value, publicKey.pkey);

  SecretService.createSecret({
    name: secretName.value,
    objectType: SecretObjectType.Credential,
    kind,
    crypted,
    keyPairId: publicKey.id,
    version: SecretVersion.V1,
    algorithm: SecretAlgorithm.Sealedbox,
  }).subscribe((response) => {
    if (response.error) {
      GlobalErrorService.set(response);
    }
    clear();
    emit("submit");
  });
};

watch(secretKind, (_current, _old) => {
  // Delete all entries in `secretMessage` as they may not apply to other
  // secret kinds and should *not* persist
  for (const key of Object.keys(secretMessage.value)) {
    delete secretMessage.value[key];
  }
});

const getPublicKey = async (): Promise<{
  id: number;
  pkey: Uint8Array;
} | null> => {
  const reply = await SecretService.getPublicKeyRaw();
  if (reply.error) {
    GlobalErrorService.set(reply);
    return null;
  } else {
    return { id: reply.id, pkey: Base64.toUint8Array(reply.public_key) };
  }
};

const encryptMessage = (
  message: Record<string, string>,
  publicKey: Uint8Array,
): number[] => {
  return Array.from(sealedBox.seal(serializeMessage(message), publicKey));
};

const serializeMessage = (message: Record<string, string>): Uint8Array => {
  const json = JSON.stringify(message, null, 0);
  const result = new Uint8Array(json.length);
  for (let i = 0; i < json.length; i++) {
    result[i] = json.charCodeAt(i);
  }
  return result;
};
</script>

<style scoped>
.background {
  background-color: #1e1e1e;
}

.header {
  background-color: #3a3d40;
}

.row-item {
  background-color: #262626;
}

.row-item:nth-child(odd) {
  background-color: #2c2c2c;
}

.table-border {
  border-bottom: 1px solid #46494d;
}
</style>
