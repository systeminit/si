<template>
  <div class="flex flex-col w-full overflow-hidden">
    <div
      class="flex flex-row items-center h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div v-if="schemaVariantId" class="text-lg">
        SchemaVariantId: {{ schemaVariantId }}
      </div>
    </div>

    <div class="flex text-sm">
      <div v-if="inputSockets && inputSockets.len() > 0">
        <div v-for="inputSocket in inputSockets" :key="inputSocket.id">
          <div class="flex flex-row row-item">
            {{ inputSocket }}
          </div>
        </div>
      </div>
      <div v-else>No InputSockets found! Huzzah.</div>
    </div>

    <div class="flex text-sm">
      <div v-if="outputSockets && outputSockets.len() > 0">
        <div v-for="outputSocket in outputSockets" :key="outputSocket.id">
          <div class="flex flex-row row-item">
            {{ outputSocket }}
          </div>
        </div>
      </div>
      <div v-else>No OutputSockets found! Huzzah.</div>
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
import { InputSocketService } from "@/service/input_socket";
import { InputSocket } from "@/api/sdf/dal/input_socket";
import { OutputSocket } from "@/api/sdf/dal/output_socket";
import { OutputSocketService } from "@/service/output_socket";

const props = defineProps<{
  schemaVariantId: number;
}>();
const { schemaVariantId } = toRefs(props);
const schemaVariantId$ = fromRef<number>(schemaVariantId, { immediate: true });

const inputSockets = refFrom<InputSocket[] | undefined>(
  Rx.combineLatest([
    schemaVariantId$,
    standardVisibilityTriggers$,
    editSessionWritten$,
  ]).pipe(
    Rx.switchMap(([schemaVariantId, [visibility]]) => {
      return InputSocketService.listInputSockets({
        schemaVariantId: schemaVariantId,
        ...visibility,
      });
    }),
    Rx.switchMap((response) => {
      if (response === null) {
        return Rx.from([[]]);
      } else if (response.error) {
        GlobalErrorService.set(response);
        return Rx.from([[]]);
      } else {
        return Rx.from([response.inputSockets]);
      }
    }),
  ),
);

const outputSockets = refFrom<OutputSocket[] | undefined>(
  Rx.combineLatest([
    schemaVariantId$,
    standardVisibilityTriggers$,
    editSessionWritten$,
  ]).pipe(
    Rx.switchMap(([schemaVariantId, [visibility]]) => {
      return OutputSocketService.listOutputSockets({
        schemaVariantId: schemaVariantId,
        ...visibility,
      });
    }),
    Rx.switchMap((response) => {
      if (response === null) {
        return Rx.from([[]]);
      } else if (response.error) {
        GlobalErrorService.set(response);
        return Rx.from([[]]);
      } else {
        return Rx.from([response.inputSockets]);
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
