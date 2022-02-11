<template>
  <div v-if="componentId" class="flex flex-col w-full overflow-hidden">
    <div
      class="flex flex-row items-center h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div v-if="componentMetadata?.schemaName" class="text-lg">
        {{ componentMetadata.schemaName }}
      </div>
    </div>
    <EditFormComponent :object-id="props.componentId" />
  </div>
</template>

<script setup lang="ts">
import EditFormComponent from "@/organisims/EditFormComponent.vue";
import { ComponentService } from "@/service/component";
import { GetComponentMetadataResponse } from "@/service/component/get_component_metadata";
import { toRefs } from "vue";
import { fromRef, refFrom } from "vuse-rx";
import { from, combineLatest } from "rxjs";
import { switchMap } from "rxjs/operators";
import { GlobalErrorService } from "@/service/global_error";

const props = defineProps<{
  componentId: number;
}>();

const { componentId } = toRefs(props);

// We need an observable stream of props.componentId. We also want
// that stream to emit a value immediately (the first value, as well as all
// subsequent values)
const componentId$ = fromRef<number>(componentId, { immediate: true });

const componentMetadata = refFrom<GetComponentMetadataResponse | undefined>(
  combineLatest([componentId$]).pipe(
    switchMap(([componentId]) => {
      return ComponentService.getComponentMetadata({
        componentId,
      });
    }),
    switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return from([undefined]);
      } else {
        return from([response]);
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

.property-section-bg-color {
  background-color: #292c2d;
}
</style>
