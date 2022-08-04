<template>
  <div
    v-if="emptyViewer"
    class="flex flex-row items-center text-center w-full h-full"
  >
    <p class="w-full text-3xl text-neutral-500">No Component Selected</p>
  </div>
  <div v-else class="w-full h-full flex flex-col">
    <div class="border rounded-xl text-center text-2xl mt-4 px-5 py-2">
      <!-- let's make this into a molecule/atom to reuse it! -->
      <CheckCircleIcon class="text-success-500" :class="iconClasses" />
      <XCircleIcon class="text-destructive-500" :class="iconClasses" />
      <ClockIcon class="text-warning-500" :class="iconClasses" />
      <!-- yeah -->
      <span class="align-middle">{{ props.componentName }}</span>
    </div>
    <div class="w-full overflow-y-auto">
      <QualificationView
        v-for="(qualification, index) in qualificationList.value"
        :key="index"
        :qualification="qualification"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { listQualifications } from "@/service/component/list_qualifications";
import { refFrom } from "vuse-rx";
import { map } from "rxjs";
import { GlobalErrorService } from "@/service/global_error";
import { computed } from "vue";
import QualificationView from "@/organisms/QualificationView.vue";
import {
  CheckCircleIcon,
  ClockIcon,
  XCircleIcon,
} from "@heroicons/vue/outline";
import component from "*.vue";

const props = defineProps<{
  componentId: number | false;
  componentName: string | false;
}>();

const iconClasses = "w-8 inline align-middle mr-1";

const emptyViewer = computed(() => props.componentId === false);

const qualificationList = computed(() => {
  if (emptyViewer.value) return [];
  return refFrom(
    listQualifications({ componentId: props.componentId as number }).pipe(
      map((response) => {
        if (response.error) {
          GlobalErrorService.set(response);
          return [];
        }
        return response;
      }),
    ),
  );
});
</script>
