<template>
  <div class="flex w-full h-full">
    <SiError
      testId="secret-list-error"
      :message="errorMessage"
      @clear="clearErrorMessage"
    />
    <div class="flex w-full h-full">
      <div class="flex flex-col w-full table-fixed shadow-sm">
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
import _ from "lodash";
import SiError from "@/atoms/SiError.vue";
import { SecretKind, SecretObjectType, ISecret } from "@/api/sdf/model/secret";
import { workspace$, refreshSecretList$ } from "@/observables";
import { from, combineLatest } from "rxjs";
import { SecretDal } from "@/api/sdf/dal/secretDal";
import { switchMap, tap } from "rxjs/operators";

interface IData {
  errorMessage: string;
  isLoading: boolean;
  secretList: ISecret[];
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
      secretList: [],
    };
  },
  subscriptions(): Record<string, any> {
    return {
      getSecretList: combineLatest(workspace$, refreshSecretList$).pipe(
        switchMap(([workspace, _refreshList]) => {
          // @ts-ignore
          this.isLoading = true;
          if (workspace) {
            return from(SecretDal.listSecrets({ workspaceId: workspace.id }));
          } else {
            return from([
              {
                error: {
                  code: 42,
                  message: "cannot fetch secret list without a workspace",
                },
              },
            ]);
          }
        }),
        tap(reply => {
          // @ts-ignore
          this.isLoading = false;
          if (reply.error) {
            if (reply.error.code != 42) {
              // @ts-ignore
              this.errorMessage = reply.error.message;
            }
          } else {
            // @ts-ignore
            this.secretList = _.sortBy(reply.list, ["name"]);
          }
        }),
      ),
    };
  },
  methods: {
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
