<template>
  <div class="flex flex-col w-full">
    <SiError
      testId="application-create-error-message"
      :message="errorMessage"
      :success="applicationCreateSuccess"
      @clear="clearErrorMessage"
    />
    <div class="flex flex-row items-center w-full pb-2">
      <div class="pr-2 text-sm text-right text-gray-400 align-middle">
        <label for="applicationName">Name:</label>
      </div>
      <div class="w-full align-middle">
        <SiTextBox
          name="applicationName"
          placeholder="super dope"
          id="applicationName"
          required
          size="sm"
          v-model="form.applicationName"
        />
      </div>
    </div>
    <div class="flex justify-end w-full">
      <div class="pr-2">
        <SiButton
          @click.native="cancel"
          size="xs"
          label="Cancel"
          kind="cancel"
          icon="null"
        />
      </div>
      <div>
        <SiButton
          @click.native="create"
          size="xs"
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
import { mapState } from "vuex";

import SiButton from "@/atoms/SiButton.vue";
import SiError from "@/atoms/SiError.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";

import {
  IApplicationCreateReply,
  IApplicationCreateRequest,
} from "@/store/modules/application";
import { SessionStore } from "@/store/modules/session";

interface IData {
  errorMessage: string;
  applicationCreateSuccess: boolean;
  form: {
    applicationName: string;
  };
}

export default Vue.extend({
  name: "ApplicationCreate",
  components: {
    SiButton,
    SiTextBox,
    SiError,
  },
  data(): IData {
    return {
      errorMessage: "",
      applicationCreateSuccess: false,
      form: {
        applicationName: "",
      },
    };
  },
  computed: {
    ...mapState({
      currentWorkspace: (state: any): SessionStore["currentWorkspace"] =>
        state.session.currentWorkspace,
      currentSystem: (state: any): SessionStore["currentSystem"] =>
        state.session.currentSystem,
    }),
  },
  methods: {
    cancel() {
      this.form.applicationName = "";
      this.applicationCreateSuccess = false;
      this.$emit("cancel");
    },
    async create() {
      this.applicationCreateSuccess = false;
      if (!this.currentWorkspace) {
        this.errorMessage = "No workspace selected!";
        return;
      }
      //if (!this.currentSystem) {
      //  this.errorMessage = "No system selected!";
      //  return;
      //}
      let request: IApplicationCreateRequest = {
        applicationName: this.form.applicationName,
        workspaceId: this.currentWorkspace.id,
        //  systemId: this.currentSystem.id,
      };
      let reply = await this.$store.dispatch(
        "application/createApplication",
        request,
      );
      if (reply.error) {
        this.errorMessage = reply.error.message;
      } else {
        this.applicationCreateSuccess = true;
        this.$emit("submit");
      }
    },
    clearErrorMessage() {
      this.errorMessage = "";
    },
  },
  async created() {
    await this.$store.dispatch("application/activate", "ApplicationCreate");
  },
  async beforeDestroy() {
    await this.$store.dispatch("application/deactivate", "ApplicationCreate");
  },
});
</script>
