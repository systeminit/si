<template>
  <div class="flex w-full h-full">
    <div class="flex w-full h-full">
      <div class="flex flex-col w-full shadow-sm table-fixed">
        <div class="flex w-full text-sm font-medium text-gray-200 header">
          <div class="w-6/12 px-2 py-1 text-center align-middle table-border">
            Name
          </div>
          <div class="w-3/12 px-2 py-1 text-center table-border">Kind</div>
          <div class="w-3/12 px-2 py-1 text-center table-border">Type</div>
        </div>

        <div class="flex flex-col text-xs text-gray-300">
          <div
            v-for="secret in secrets"
            :key="secret.id"
            class="flex items-center row-item"
          >
            <div class="w-6/12 px-2 py-1 text-center">
              {{ secret.name }}
            </div>
            <div class="w-3/12 px-2 py-1 text-center">
              {{ secret.kind }}
            </div>
            <div class="w-3/12 px-2 py-1 text-center">
              {{ secret.objectType }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { refFrom } from "vuse-rx";
import { Secret } from "@/api/sdf/dal/secret";
import { SecretService } from "@/service/secret";
import { from, switchMap } from "rxjs";
import { GlobalErrorService } from "@/service/global_error";

const secrets = refFrom<Secret[] | undefined>(
  SecretService.listSecrets().pipe(
    switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return from([undefined]);
      } else {
        return from([response.list]);
      }
    }),
  ),
);
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
