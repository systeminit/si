<template>
  <div class="flex flex-col w-full overflow-hidden">
    <div
      class="flex flex-row items-center h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div class="text-lg">Providers</div>
    </div>

    <div class="flex text-sm p-6 flex-col">
      <div class="font-semibold">InternalProviders</div>
      <div v-if="allProviders && allProviders.internalProviders.length > 0">
        <div
          v-for="internalProvider in allProviders.internalProviders"
          :key="internalProvider.id"
        >
          <div class="flex flex-row row-item">
            {{ internalProvider }}
          </div>
        </div>
      </div>
      <div v-else class="flex">None</div>
    </div>

    <div class="flex text-sm p-6 flex-col">
      <div class="font-semibold">ExternalProviders</div>
      <div v-if="allProviders && allProviders.externalProviders.length > 0">
        <div
          v-for="externalProvider in allProviders.externalProviders"
          :key="externalProvider.id"
        >
          <div class="flex flex-row row-item">
            {{ externalProvider }}
          </div>
        </div>
      </div>
      <div v-else class="flex">None</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import * as Rx from "rxjs";
import { toRefs } from "vue";
import { fromRef, refFrom } from "vuse-rx";
import { GlobalErrorService } from "@/service/global_error";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { editSessionWritten$ } from "@/observable/edit_session";
import { ProviderService } from "@/service/provider";
import { ListAllProviderResponse } from "@/service/provider/list_all_providers";

const props = defineProps<{
  schemaVariantId: number;
}>();

const { schemaVariantId } = toRefs(props);
const schemaVariantId$ = fromRef<number>(schemaVariantId, { immediate: true });

const allProviders = refFrom<ListAllProviderResponse | undefined>(
  Rx.combineLatest([
    schemaVariantId$,
    standardVisibilityTriggers$,
    editSessionWritten$,
  ]).pipe(
    Rx.switchMap(([schemaVariantId, [visibility]]) => {
      return ProviderService.listAllProviders({
        schemaVariantId: schemaVariantId,
        ...visibility,
      });
    }),
    Rx.switchMap((response) => {
      if (response === null) {
        return Rx.from([undefined]);
      } else if (response.error) {
        GlobalErrorService.set(response);
        return Rx.from([undefined]);
      } else {
        return Rx.from([response]);
      }
    }),
  ),
);
</script>

<style scoped>
.scrollbar {
  -ms-overflow-style: none; /* edge, and ie */
  scrollbar-width: none; /* firefox */
}
.scrollbar::-webkit-scrollbar {
  display: none; /*chrome, opera, and safari */
}
.gold-bars-icon {
  color: #ce7f3e;
}
.property-section-bg-color {
  background-color: #292c2d;
}
.ok {
  color: #86f0ad;
}
.warning {
  color: #f0d286;
}
.error {
  color: #f08686;
}
.unknown {
  color: #5b6163;
}
</style>
