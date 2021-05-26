<template>
  <div class="flex flex-col w-full">
    <SiError
      testId="secret-create-error-message"
      :message="errorMessage"
      :success="createWasSuccessful"
      @clear="clearErrorMessage"
    />
    <div class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-sm text-right text-gray-400 align-middle">
        <label for="secretName">Secret Name:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          size="xs"
          name="secretName"
          placeholder="secret name"
          :isShowType="false"
          id="secretName"
          required
          v-model="form.secretName"
        />
      </div>
    </div>
    <div class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-sm text-right text-gray-400 align-middle">
        <label for="secretKind">Secret Kind:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiSelect
          size="xs"
          name="secretKind"
          id="secretKind"
          :options="secretKinds"
          v-model="form.secretKind"
          required
        />
      </div>
    </div>

    <AwsAccessKeyCredential @input="updateMessage" v-if="kindIsAwsAccesKey" />
    <DockerHubCredential @input="updateMessage" v-else-if="kindIsDockerHub" />
    <HelmRepoCredential @input="updateMessage" v-else-if="kindIsHelmRepo" />
    <AzureServicePrincipal
      @input="updateMessage"
      v-else-if="kindIsAzureServicePrincipal"
    />

    <div class="flex justify-end w-full">
      <div class="pr-2">
        <SiButton
          size="xs"
          @click.native="cancel"
          label="Cancel"
          kind="cancel"
          icon="null"
        />
      </div>
      <div>
        <SiButton
          size="xs"
          @click.native="create"
          label="Create"
          kind="save"
          icon="null"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import SiButton from "@/atoms/SiButton.vue";
import SiError from "@/atoms/SiError.vue";
import SiSelect, { SelectProps } from "@/atoms/SiSelect.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import {
  SecretKind,
  SecretVersion,
  SecretAlgorithm,
} from "@/api/sdf/model/secret";
import AwsAccessKeyCredential from "@/organisims/SecretCreate/AwsAccessKeyCredential.vue";
import DockerHubCredential from "@/organisims/SecretCreate/DockerHubCredential.vue";
import HelmRepoCredential from "@/organisims/SecretCreate/HelmRepoCredential.vue";
import AzureServicePrincipal from "@/organisims/SecretCreate/AzureServicePrincipal.vue";
import { workspace$, refreshSecretList$ } from "@/observables";
import { SecretDal, ICreateSecretRequest } from "@/api/sdf/dal/secretDal";
import sealedBox from "tweetnacl-sealedbox-js";

interface IData {
  form: {
    secretName: string;
    secretKind: SecretKind | null;
    message: Record<string, string>;
  };
  createWasSuccessful: boolean;
  errorMessage: string;
}

export function encryptMessage(
  message: Record<string, string>,
  publicKey: Uint8Array,
): number[] {
  return Array.from(sealedBox.seal(serializeMessage(message), publicKey));
}

export function serializeMessage(message: Record<string, string>): Uint8Array {
  const json = JSON.stringify(message, null, 0);

  const result = new Uint8Array(json.length);
  for (let i = 0; i < json.length; i++) {
    result[i] = json.charCodeAt(i);
  }

  return result;
}

export default Vue.extend({
  name: "SecretCreate",
  components: {
    AwsAccessKeyCredential,
    DockerHubCredential,
    HelmRepoCredential,
    AzureServicePrincipal,
    SiButton,
    SiError,
    SiSelect,
    SiTextBox,
  },
  data(): IData {
    return {
      form: {
        secretName: "",
        secretKind: null,
        message: {},
      },
      createWasSuccessful: false,
      errorMessage: "",
    };
  },
  subscriptions(): Record<string, any> {
    return {
      currentWorkspace: workspace$,
    };
  },
  computed: {
    secretKinds(): SelectProps["options"] {
      let secretKinds = SecretKind.selectPropOptions();
      secretKinds.unshift({ label: "", value: "" });
      return secretKinds;
    },
    kindIsAzureServicePrincipal(): boolean {
      return this.form.secretKind == SecretKind.AzureServicePrincipal;
    },
    kindIsAwsAccesKey(): boolean {
      return this.form.secretKind == SecretKind.AwsAccessKey;
    },
    kindIsDockerHub(): boolean {
      return this.form.secretKind == SecretKind.DockerHub;
    },
    kindIsHelmRepo(): boolean {
      return this.form.secretKind == SecretKind.HelmRepo;
    },
  },
  methods: {
    updateMessage(event: Record<string, any>) {
      this.form.message = event;
    },
    clear() {
      this.form.secretName = "";
      this.form.secretKind = null;
      this.form.message = {};

      this.createWasSuccessful = false;
      this.clearErrorMessage();
    },
    cancel() {
      this.clear();
      this.$emit("cancel");
    },
    clearErrorMessage() {
      this.errorMessage = "";
    },
    async create() {
      // empty out the error message, ready for this attempt
      this.errorMessage = "";

      // @ts-ignore
      if (!this.currentWorkspace) {
        this.errorMessage = "No workspace selected!";
        return;
      }
      if (!this.form.secretKind) {
        this.errorMessage = "No secret kind selected!";
        return;
      }

      const pkReply = await SecretDal.getPublicKey();
      if (pkReply.error) {
        return pkReply;
      }
      const publicKey = pkReply.publicKey;

      const crypted = encryptMessage(this.form.message, publicKey.publicKey);

      const dalRequest: ICreateSecretRequest = {
        name: this.form.secretName,
        objectType: SecretKind.objectTypeFor(this.form.secretKind),
        kind: this.form.secretKind,
        crypted,
        keyPairId: publicKey.id,
        version: SecretVersion.defaultValue(),
        algorithm: SecretAlgorithm.defaultValue(),
        // @ts-ignore
        workspaceId: this.currentWorkspace.id,
      };

      const reply = await SecretDal.createSecret(dalRequest);

      if (reply.error) {
        this.errorMessage = reply.error.message;
      } else {
        this.createWasSuccessful = true;
        this.clear();
        refreshSecretList$.next(true);
        this.$emit("submit");
      }
    },
  },
});
</script>
