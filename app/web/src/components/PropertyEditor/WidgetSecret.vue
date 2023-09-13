<template>
  <div
    v-if="featureFlagsStore.SECRETS"
    class="flex flex-col items-center pt-sm pl-lg pr-md"
  >
    <div class="text-sm font-medium w-full pb-xs">Secret: {{ name }}</div>
    <div class="flex flex-row items-center w-full">
      <div class="flex flex-col grow">
        <VButton label="Select Secret" @click="(e) => popoverRef.open(e)" />
        <Popover ref="popoverRef" anchorDirectionX="left" anchorAlignY="bottom">
          <SecretsList definitionId="Mocks" />
        </Popover>
      </div>
      <div class="pl-sm">
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
import Popover from "@/components/Popover.vue";
import SecretsList from "@/components/SecretsList.vue";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import {
  PropertyEditorValidation,
  PropertyPath,
  UpdatedProperty,
} from "@/api/sdf/dal/property_editor";
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
</script>
