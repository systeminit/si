<template>
  <template v-if="props.coreEditField">
    <div v-show="show" class="flex flex-row items-center mx-6 mt-2">
      <div class="text-sm leading-tight text-right text-white w-28">
        <slot name="name"></slot>
      </div>
      <div
        v-if="editMode"
        class="flex flex-grow pl-2 text-sm leading-tight text-gray-400"
        @keyup.stop
        @keydown.stop
      >
        <slot name="edit" />
      </div>
      <div
        v-else
        class="flex flex-grow pl-2 mr-2 text-sm leading-tight text-gray-400"
      >
        <slot name="show"></slot>
      </div>
    </div>
  </template>
  <template v-else>
    <div v-show="show" class="flex flex-row items-center w-full">
      <div class="flex flex-col w-full">
        <div class="flex flex-row items-center">
          <div
            class="flex-wrap self-start text-sm leading-tight text-right w-36"
          >
            <slot name="name"></slot>
          </div>
          <div class="flex w-full">
            <div
              v-if="editMode"
              class="flex mx-2 text-sm leading-tight text-gray-400"
              @keyup.stop
              @keydown.stop
            >
              <slot name="edit" />
            </div>

            <div v-else class="flex mx-2 text-sm leading-tight text-gray-400">
              <!-- could flex-grow if needed -->
              <slot name="show" />
            </div>
          </div>
        </div>

        <div class="flex flex-wrap">
          <ValidationErrorsWidget
            :errors="props.validationErrors"
            class="p-2"
          />
        </div>
      </div>
    </div>
  </template>
</template>

<script setup lang="ts">
import { refFrom } from "vuse-rx";
import { ChangeSetService } from "@/service/change_set";
import ValidationErrorsWidget from "@/organisims/EditForm/ValidationErrorsWidget.vue";
import type { ValidationErrors } from "@/api/sdf/dal/edit_field";

const props = defineProps<{
  show: boolean;
  coreEditField: boolean;
  validationErrors: ValidationErrors;
}>();

const editMode = refFrom(ChangeSetService.currentEditMode());
</script>
