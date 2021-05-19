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

import SiButton from "@/atoms/SiButton.vue";
import SiError from "@/atoms/SiError.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";

import { workspace$, system$, applicationCreated$ } from "@/observables";
import {
  ApplicationDal,
  IApplicationCreateRequest,
} from "@/api/sdf/dal/applicationDal";

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
  subscriptions(): Record<string, any> {
    return {
      currentWorkspace: workspace$,
      currentSystem: system$,
    };
  },
  methods: {
    cancel() {
      this.form.applicationName = "";
      this.applicationCreateSuccess = false;
      this.$emit("cancel");
    },
    async create(this: any) {
      this.applicationCreateSuccess = false;
      if (!this.currentWorkspace) {
        this.errorMessage = "No workspace selected!";
        return;
      }
      let request: IApplicationCreateRequest = {
        applicationName: this.form.applicationName,
        workspaceId: this.currentWorkspace.id,
      };
      let reply = await ApplicationDal.createApplication(request);
      if (reply.error) {
        this.errorMessage = reply.error.message;
      } else {
        this.applicationCreateSuccess = true;
        applicationCreated$.next(reply);
        this.$emit("submit");
      }
    },
    clearErrorMessage() {
      this.errorMessage = "";
    },
  },
});
</script>
