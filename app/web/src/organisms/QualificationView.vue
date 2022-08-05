<template>
  <Disclosure as="div" class="border rounded-xl text-left">
    <disclosure-button
      v-slot="{ open }"
      as="h1"
      class="text-xl py-2 px-5 flex justify-between"
    >
      <div>
        <StatusIndicatorIcon :status="qualificationStatus" class="w-6 mr-1" />
        <span class="align-middle">{{ qualification.title }}</span>
      </div>
      <ChevronUpIcon
        :class="open ? 'rotate-180 transform' : ''"
        class="w-5 dark:text-white inline-block align-middle transition-all"
      />
    </disclosure-button>

    <disclosure-panel as="div" class="py-2 px-5">
      <div class="w-full flex justify-between text-neutral-500 min-h-4">
        <p>{{ qualification.description ?? "No description" }}</p>
        <a :href="qualification.link" target="_blank">
          {{ qualification.link }}
        </a>
      </div>

      <p
        v-if="!qualification.output.length"
        class="text-neutral-500 text-center"
      >
        No Output
      </p>

      <p
        v-for="(output, index) in qualification.output"
        v-else
        :key="index"
        class="text-sm"
      >
        {{ output }}
      </p>
    </disclosure-panel>
  </Disclosure>
</template>

<script lang="ts" setup>
import { Qualification } from "@/api/sdf/dal/qualification";
import { Disclosure, DisclosureButton, DisclosurePanel } from "@headlessui/vue";
import StatusIndicatorIcon, {
  Status,
} from "@/molecules/StatusIndicatorIcon.vue";
import { ChevronUpIcon } from "@heroicons/vue/solid";
import { computed, ComputedRef } from "vue";
import _ from "lodash";

const props = defineProps<{
  qualification: Qualification;
}>();

const qualificationStatus: ComputedRef<Status> = computed(() => {
  if (_.isNil(props.qualification.result)) return "loading";

  if (props.qualification.result.success) return "success";

  return "failure";
});
</script>
