<template>
  <div class="w-full h-full flex flex-col">
    <div class="overflow-y-auto flex flex-row mt-4 mx-2 flex-wrap">
      <!-- Note(victor): The only reason there's this extra Div here is to allow us to have margins between -->
      <!-- QualificationViews while using flex-basis to keep stuff responsive. We should revisit this and tune -->
      <!-- the breakpoints after the content and design of the View is solidified -->
      <div
        v-for="(qualification, index) in qualificationList"
        :key="index"
        class="basis-full lg:basis-1/2 xl:basis-1/3 overflow-hidden pb-4 px-2"
      >
        <QualificationViewerSingle :qualification="qualification" />
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
import QualificationViewerSingle from "@/organisms/StatusBarTabs/Qualification/QualificationViewerSingle.vue";
import { Status } from "@/molecules/StatusIndicatorIcon.vue";
import { switchMap } from "rxjs/operators";
import { Qualification } from "@/api/sdf/dal/qualification";

const props = defineProps<{
  componentId: number;
  componentName: string;
  componentQualificationStatus: Status;
}>();

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
