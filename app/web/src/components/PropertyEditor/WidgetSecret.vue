<template>
  <div
    v-if="featureFlagsStore.SECRETS"
    class="flex flex-col items-center pt-sm pl-lg pr-sm"
  >
    <div class="text-sm font-medium w-full pb-xs">Secret: {{ name }}</div>
    <div class="flex flex-row items-center w-full">
      <div class="flex flex-col grow">
        <div
          v-if="value"
          :class="
            clsx(
              'sm:text-sm font-bold grow p-xs block',
              'border rounded-sm shadow-sm focus:outline-none',
              'bg-neutral-50 dark:border-neutral-600 dark:bg-neutral-900 border-neutral-600 cursor-pointer',
              'hover:border-action-500 hover:outline-action-500 hover:outline -outline-offset-1',
            )
          "
          @click="(e) => popoverRef.open(e)"
        >
          {{ value }}
        </div>
        <VButton
          v-else
          label="Select Secret"
          @click="(e) => popoverRef.open(e)"
        />
        <Popover ref="popoverRef" anchorDirectionX="left" anchorAlignY="bottom">
          <SecretsList definitionId="Mocks" @select="setField" />
        </Popover>
      </div>
      <div v-if="value" class="pl-xs">
        <SiButtonIcon
          v-if="!disabled"
          tooltipText="Unset field"
          icon="x-circle"
          :disabled="disableUnset"
          @click="unsetField"
        />
      </div>
    </div>
    <div v-if="docLink" class="w-full mt-2 text-xs text-action-500">
      <a :href="docLink" target="_blank" class="hover:underline">
        Documentation
      </a>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, toRefs } from "vue";
import { VButton } from "@si/vue-lib/design-system";
import clsx from "clsx";
import Popover from "@/components/Popover.vue";
import SecretsList from "@/components/SecretsList.vue";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import {
  PropertyEditorValidation,
  PropertyPath,
  UpdatedProperty,
} from "@/api/sdf/dal/property_editor";
import { Secret } from "@/store/secrets.store";
import SiButtonIcon from "../SiButtonIcon.vue";

const featureFlagsStore = useFeatureFlagsStore();

const popoverRef = ref();

const props = defineProps<{
  name: string;
  path?: PropertyPath;
  collapsedPaths: Array<Array<string>>;
  value: unknown;
  propId: string;
  valueId: string;
  validation?: PropertyEditorValidation;
  docLink?: string;
  disabled?: boolean;
  required?: boolean;
  description?: string;
}>();

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const { name, path, collapsedPaths, valueId, propId, value } = toRefs(props);

const disableUnset = computed(() => {
  if ((value.value ?? null) === null) {
    return true;
  } else {
    return false;
  }
});

const emit = defineEmits<{
  (e: "updatedProperty", v: UpdatedProperty): void;
}>();

const unsetField = () => {
  emit("updatedProperty", {
    value: null,
    propId: propId.value,
    valueId: valueId.value,
  });
};

const setField = (secret: Secret) => {
  popoverRef.value.close();
  emit("updatedProperty", {
    value: secret,
    propId: propId.value,
    valueId: valueId.value,
  });
};
</script>
