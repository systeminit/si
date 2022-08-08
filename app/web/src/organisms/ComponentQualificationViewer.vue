<template>
  <div class="w-full h-full flex flex-col">
    <div class="border rounded-xl text-center text-2xl mt-4 mx-2 py-2">
      <StatusIndicatorIcon
        :status="componentQualificationStatus"
        class="w-8 mr-1"
      />
      <span class="align-middle">{{ props.componentName }}</span>
    </div>
    <div class="overflow-y-auto flex flex-row mt-4 mx-2 flex-wrap">
      <!-- Note(victor): The only reason there's this extra Div here is to allow us to have margins between -->
      <!-- QualificationViews while using flex-basis to keep stuff responsive. We should revisit this and tune -->
      <!-- the breakpoints after the content and design of the View is solidified -->
      <div
        v-for="(qualification, index) in qualificationList"
        :key="index"
        class="basis-full lg:basis-1/2 xl:basis-1/3"
      >
        <QualificationView :qualification="qualification" class="mb-4 mx-2" />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import {
  listQualifications,
  ListQualificationsResponse,
} from "@/service/component/list_qualifications";
import { fromRef, refFrom } from "vuse-rx";
import { map } from "rxjs";
import { GlobalErrorService } from "@/service/global_error";
import QualificationView from "@/organisms/QualificationView.vue";
import StatusIndicatorIcon, {
  Status,
} from "@/molecules/StatusIndicatorIcon.vue";
import { switchMap } from "rxjs/operators";
import { ref } from "vue";
import { Qualification } from "@/api/sdf/dal/qualification";

const props = defineProps<{
  componentId: number;
  componentName: string;
}>();

const componentQualificationStatus = ref<Status>("success"); // TODO(victor): This should be received from listQualifications, probably

const qualificationList = refFrom<Array<Qualification>>(
  fromRef(props, { immediate: true }).pipe(
    switchMap(({ componentId }) =>
      listQualifications({ componentId }).pipe(
        map((response) => {
          if (response.error) {
            GlobalErrorService.set(response);
            return [];
          }
          return response as ListQualificationsResponse;
        }),
      ),
    ),
  ),
);
</script>
