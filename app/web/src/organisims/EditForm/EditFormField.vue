<template>
  <div v-if="show" class="flex flex-row items-center w-full">
    <div class="flex flex-col w-full">
      <div class="flex flex-row items-center">
        <div class="flex-wrap self-start text-sm leading-tight text-right w-36">
          <slot name="name"></slot>
        </div>
        <div class="flex w-full">
          <div
            v-if="editMode"
            class="flex mx-2 text-sm leading-tight text-gray-400"
            @keyup.stop
            @keydown.stop
          >
            <!-- could flex-grow if needed -->
            <slot name="edit" />
          </div>

          <div v-else class="flex mx-2 text-sm leading-tight text-gray-400">
            <!-- could flex-grow if needed -->
            <slot name="show" />
          </div>
        </div>
      </div>
      <div class="flex flex-wrap">
        <ValidationErrors :errors="errors" class="p-2" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { refFrom } from "vuse-rx";
import { ChangeSetService } from "@/service/change_set";
import ValidationErrors from "@/organisims/EditForm/ValidationErrors.vue";

defineProps({
  show: {
    type: Boolean,
    required: true,
  },
});
const editMode = refFrom(ChangeSetService.currentEditMode());
const errors = ref([{ message: "Aieeee!", link: "https://placekitten.com" }]);
</script>

<style scoped></style>
