<template>
  <div class="flex w-full h-full">
    <SiError
      testId="secret-list-error"
      :message="errorMessage"
      @clear="clearErrorMessage"
    />
    <div class="flex w-full h-full">
      <div class="flex flex-col w-full shadow-sm table-fixed">
        <div class="flex w-full text-sm font-medium text-gray-200 header">
          <div class="w-8/12 px-2 py-1 text-center align-middle table-border">
            Name
          </div>
          <div class="w-2/12 px-2 py-1 text-center table-border">
            Kind
          </div>
          <div class="w-2/12 px-2 py-1 text-center table-border">
            Type
          </div>
        </div>

        <div class="flex flex-col overflow-y-scroll text-xs text-gray-300">
          <div
            class="flex items-center row"
            v-for="secret in secretList"
            :key="secret.id"
          >
            <div class="w-8/12 px-2 py-1 text-center">
              {{ secret.name }}
            </div>
            <div class="w-2/12 px-2 py-1 text-center ">
              {{ labelForKind(secret.kind) }}
            </div>
            <div class="w-2/12 px-2 py-1 text-center ">
              {{ labelForObjectType(secret.objectType) }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState } from "vuex";
import _ from "lodash";
import { SecretStore, ISetSecretListReply } from "@/store/modules/secret";
import SiError from "@/atoms/SiError.vue";
import { SessionStore } from "@/store/modules/session";
import { SecretKind, SecretObjectType } from "@/api/sdf/model/secret";

interface IData {
  errorMessage: string;
  isLoading: boolean;
}

export default Vue.extend({
  name: "SecretList",
  components: {
    SiError,
  },
  data(): IData {
    return {
      isLoading: true,
      errorMessage: "",
    };
  },
  computed: {
    ...mapState({
      secretList(state: Record<string, any>): SecretStore["secretList"] {
        return _.sortBy(state.secret.secretList, ["name"]);
      },
      currentWorkspace: (state: any): SessionStore["currentWorkspace"] =>
        state.session.currentWorkspace,
    }),
  },
  methods: {
    async setSecretList(
      workspace: SessionStore["currentWorkspace"],
    ): Promise<void> {
      if (workspace) {
        this.isLoading = true;
        const reply: ISetSecretListReply = await this.$store.dispatch(
          "secret/setSecretList",
          { workspaceId: workspace.id },
        );
        if (reply.error) {
          this.errorMessage = reply.error.message;
        } else {
          this.errorMessage = "";
        }
        this.isLoading = false;
      }
    },
    labelForKind(secretKind: SecretKind): string {
      return SecretKind.labelFor(secretKind);
    },
    labelForObjectType(secretObjectType: SecretObjectType): string {
      return SecretObjectType.labelFor(secretObjectType);
    },
    clearErrorMessage() {
      this.errorMessage = "";
    },
  },
  async created() {
    await this.setSecretList(this.currentWorkspace);
  },
  watch: {
    async currentWorkspace(workspace: SessionStore["currentWorkspace"]) {
      await this.setSecretList(workspace);
    },
  },
});
</script>

<style scoped>
.background {
  background-color: #1e1e1e;
}

.header {
  background-color: #3a3d40;
}

.row {
  background-color: #262626;
}

.row:nth-child(odd) {
  background-color: #2c2c2c;
}

.table-border {
  border-bottom: 1px solid #46494d;
}
</style>
