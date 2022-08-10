<template>
  <div class="border rounded-xl text-left py-2 px-5 h-full flex flex-col">
    <div class="text-xl flex justify-between pb-2">
      <div>
        <StatusIndicatorIcon :status="qualificationStatus" class="w-6 mr-1.5" />
        <span class="align-middle">{{ qualification.title }}</span>
      </div>
      <ExternalLinkIcon
        class="w-5 dark:text-white inline-block align-middle transition-all"
      />
    </div>

    <div class="w-full flex justify-between text-neutral-500 mb-2">
      <p>{{ qualification.description ?? "No description" }}</p>
      <a :href="qualification.link" target="_blank">
        {{ qualification.link }}
      </a>
    </div>

    <div
      class="font-commodore bg-black overflow-hidden py-1 flex-grow flex flex-col justify-center"
    >
      <p
        v-if="!qualification.output.length"
        class="text-neutral-500 text-center"
      >
        No Output
      </p>

      <p
        v-for="(output, index) in qualification.output.slice(-5)"
        v-else
        :key="index"
        class="text-sm whitespace-nowrap overflow-hidden overflow-ellipsis px-1.5"
      >
        {{ output.line }}
      </p>
    </div>
  </div>
</template>

<script lang="ts" setup>
import StatusIndicatorIcon from "@/molecules/StatusIndicatorIcon.vue";
import { ExternalLinkIcon } from "@heroicons/vue/solid";
import { computed } from "vue";
import _ from "lodash";
import { Qualification } from "@/api/sdf/dal/qualification";

const props = defineProps<{
  qualification: Qualification;
}>();

const qualificationStatus = computed(() => {
  if (_.isNil(props.qualification.result)) return "loading";

  if (props.qualification.result.success) return "success";

  return "failure";
});
</script>
