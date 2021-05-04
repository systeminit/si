<template>
  <div class="flex flex-col w-full">
    <SiError
      testId="secret-create-error-message"
      :message="errorMessage"
      :success="createWasSuccessful"
      @clear="clearErrorMessage"
    />
    <div class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-right text-gray-400 align-middle">
        <label for="secretName">Secret Name:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          name="secretName"
          placeholder="my key"
          id="secretName"
          required
          v-model="form.secretName"
        />
      </div>
    </div>
    <div class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-right text-gray-400 align-middle">
        <label for="secretKind">Secret Kind:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiSelect
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

    <div class="flex justify-end w-full">
      <div class="pr-2">
        <SiButton
          @click.native="cancel"
          label="Cancel"
          kind="cancel"
          icon="cancel"
        />
      </div>
      <div>
        <SiButton
          @click.native="create"
          label="Create"
          kind="save"
          icon="save"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState } from "vuex";
import SiButton from "@/atoms/SiButton.vue";
import SiError from "@/atoms/SiError.vue";
import SiSelect, { SelectProps } from "@/atoms/SiSelect.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import { SecretKind } from "@/api/sdf/model/secret";
import { ISecretCreateRequest } from "@/store/modules/secret";
import AwsAccessKeyCredential from "@/organisims/SecretCreate/AwsAccessKeyCredential.vue";
import DockerHubCredential from "@/organisims/SecretCreate/DockerHubCredential.vue";
import HelmRepoCredential from "@/organisims/SecretCreate/HelmRepoCredential.vue";
import { SessionStore } from "@/store/modules/session";

interface IData {
  form: {
    secretName: string;
    secretKind: SecretKind | null;
    message: Record<string, string>;
  };
  createWasSuccessful: boolean;
  errorMessage: string;
}

export default Vue.extend({
  name: "SecretCreate",
  components: {
    AwsAccessKeyCredential,
    DockerHubCredential,
    HelmRepoCredential,
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
  computed: {
    ...mapState({
      currentWorkspace: (state: any): SessionStore["currentWorkspace"] =>
        state.session.currentWorkspace,
    }),
    secretKinds(): SelectProps["options"] {
      let secretKinds = SecretKind.selectPropOptions();
      secretKinds.unshift({ label: "", value: "" });
      return secretKinds;
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
    async created() {
      console.log("boop: created");
    },
    async beforeDestroy() {
      console.log("boop: beforeDestroy");
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

      if (!this.currentWorkspace) {
        this.errorMessage = "No workspace selected!";
        return;
      }
      if (!this.form.secretKind) {
        this.errorMessage = "No secret kind selected!";
        return;
      }

      const request: ISecretCreateRequest = {
        name: this.form.secretName,
        kind: this.form.secretKind,
        message: this.form.message,
      };
      const reply = await this.$store.dispatch("secret/createSecret", request);
      if (reply.error) {
        this.errorMessage = reply.error.message;
      } else {
        this.createWasSuccessful = true;
        this.clear();
        this.$emit("submit");
      }
    },
  },
});
</script>
