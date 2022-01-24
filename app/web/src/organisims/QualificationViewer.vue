<template>
  <div v-if="componentId" class="flex flex-col w-full">
    <div class="flex">
      <div>
        <div>Component ID {{ props.componentId }} Qualifications</div>
      </div>

      <div class="flex">
        <button><VueFeather type="refresh-cw" size="1.5rem" /></button>
        <VueFeather type="check-square" size="1.5rem" />
      </div>
    </div>

    <div class="flex flex-col">
      <div>QualificationChecks Here!</div>

      <div class="flex">
        <div class="flex flex-col">
          <div
            v-for="q in allQualifications"
            :key="q.name"
            class="flex flex-col"
          >
            <div class="flex flex-row">
              <div v-if="showQualificationStarting" class="flex">
                <VueFeather
                  type="rotate-cw"
                  animation="spin"
                  animation-speed="slow"
                  size="1.5rem"
                />
              </div>
              <div v-else-if="showQualificationResult" class="flex">
                <VueFeather type="smile" color="green" size="1.5rem" />
                <VueFeather type="frown" color="red" size="1.5rem" />
              </div>
              <div v-else class="flex">
                <VueFeather type="square" size="1.5rem" />
              </div>
              <div class="flex">title: {{ q.title }}</div>
              <div v-if="showQualificationLink" class="flex">
                <a target="_blank" :href="q.link">
                  <VueFeather type="link" size="1.5rem" />
                </a>
              </div>
              <div class="flex flex-grow"></div>
            </div>

            <!-- NOTE(nick): showing description should be toggleable. -->
            <div v-if="q.description" class="flex flex-col">
              <div v-if="q.result" class="flex flex-col">
                <div class="border border-solid border-slate-100">
                  <QualificationOutput :result="q.result" />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ComponentService } from "@/service/component";
import QualificationOutput from "./QualificationViewer/QualificationOutput.vue";
import VueFeather from "vue-feather";
import { refFrom } from "vuse-rx";
import { from, switchMap } from "rxjs";
import { GlobalErrorService } from "@/service/global_error";
import { Qualification } from "@/api/sdf/dal/qualification";
//import { ListQualificationsResponse } from "@/service/component/list_qualifications";

const showQualificationResult = true;
const showQualificationStarting = true;
const showQualificationLink = true;

const props = defineProps<{
  componentId: number;
}>();

const allQualifications = refFrom<Array<Qualification>>(
  ComponentService.listQualifications({
    componentId: props.componentId,
  }).pipe(
    switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return from([[]]);
      } else {
        return from([response]);
      }
    }),
  ),
);
</script>
