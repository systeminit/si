<template>
  <div class="vld-parent">
    <SiLoader :isLoading="isLoading" />
    <SiError testId="secret-list-error" :message="errorMessage" />
    <div class="flex flex-col">
      <table class="w-full table-fixed">
        <thead>
          <tr class="text-xs text-gray-200">
            <th class="w-1/2 px-4 py-2">Name</th>
            <th class="w-1/4 px-4 py-2">Kind</th>
            <th class="w-1/4 px-4 py-2">Type</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-800" v-if="secretList.length > 0">
          <tr v-for="secret in secretList" :key="secret.id">
            <td class="px-4 py-4">{{ secret.name }}</td>
            <td class="px-4 py-4">{{ labelForKind(secret.kind) }}</td>
            <td class="px-4 py-4 text-right">
              {{ labelForObjectType(secret.objectType) }}
            </td>
          </tr>
        </tbody>
        <tbody class="divide-y divide-gray-800" v-else>
          <tr>
            <td colspan="3" class="text-center italic">
              No secrets created yet!
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState } from "vuex";
import _ from "lodash";
import { SecretStore, ISetSecretListReply } from "@/store/modules/secret";
import SiError from "@/atoms/SiError.vue";
import SiLoader from "@/atoms/SiLoader.vue";
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
    SiLoader,
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
