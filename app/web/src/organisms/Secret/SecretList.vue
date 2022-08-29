<template>
  <div class="flex w-full h-full">
    <div class="flex w-full h-full">
      <div class="flex flex-col w-full shadow-sm table-fixed">
        <div
          class="flex w-full text-sm font-medium text-neutral-200 bg-neutral-800"
        >
          <div
            class="w-6/12 px-2 py-1 text-center align-middle border border-neutral-700"
          >
            Name
          </div>
          <div class="w-3/12 px-2 py-1 text-center border border-neutral-700">
            Kind
          </div>
          <div class="w-3/12 px-2 py-1 text-center border border-neutral-700">
            Type
          </div>
        </div>

        <div class="flex flex-col text-xs text-neutral-300">
          <div
            v-for="secret in secrets"
            :key="secret.id"
            class="flex items-center bg-neutral-900 odd:bg-neutral-800"
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
import { from, switchMap } from "rxjs";
import { Secret } from "@/api/sdf/dal/secret";
import { SecretService } from "@/service/secret";
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
