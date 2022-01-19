<template>
  <div v-if="componentId" class="flex flex-col w-full">
    <div
      class="relative flex flex-row items-center justify-between h-10 pt-2 pb-2 pl-6 pr-6 text-white property-section-bg-color"
    >
      <div class="text-lg">COMPONENT_NAME Qualifications</div>
    </div>

    <div class="flex">RefreshCwIcon CheckSquareIcon</div>
  </div>

  <div class="flex flex-col mx-4 mt-2 border qualification-card">
    <div class="px-2 py-2 text-xs font-medium align-middle title">
      QualificationChecks
    </div>

    <div
      v-if="!schemaEmpty"
      class="flex w-full h-full pt-2 pb-4 overflow-auto background-color"
    >
      <div class="flex flex-col w-full">
        <div
          v-for="q in allQualifications"
          :key="q.name"
          class="flex flex-col py-1 mx-2 mt-2 text-sm border qualification-section"
        >
          <div class="flex flex-row items-center w-full pl-4 my-1">
            <div v-if="showQualificationStarting" class="flex">VueLoading</div>
            <div v-else-if="showQualificationResult" class="flex">
              SmileOrFrownIcon
            </div>
            <div v-else class="flex">SquareIcon</div>
            <div class="flex ml-2 text-xs qualification-check-title">
              QualificationCheckTitle
            </div>
            <div v-if="showQualificationLink" class="flex ml-2">
              <a target="_blank" :href="link"> InfoIcon </a>
            </div>
            <div class="flex justify-end flex-grow pr-4">
              ToggleDescriptionButton
            </div>
          </div>
        </div>

        <div v-if="showDescription[q.name]" class="flex flex-col w-full">
          <div v-if="showQualificationResult" class="flex flex-col w-full">
            <div class="mt-1">
              <QualificationOutput />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const showDescription = true;
const showQualificationResult = true;
const showQualificationStarting = true;
const showQualificationLink = true;
const schemaEmpty = true;
const link = "https://bit.ly/3qHuTNh";

const allQualifications = [
  {
    name: "foo",
  },
  {
    name: "bar",
  },
];

const props = defineProps<{
  componentId: number;
}>();
</script>
