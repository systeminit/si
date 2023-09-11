<template>
  <div
    v-if="featureFlagsStore.SECRETS"
    class="flex flex-col w-full pt-sm px-lg"
  >
    <VButton label="Select Secret" @click="(e) => popoverRef.open(e)" />
    <Popover ref="popoverRef" anchorDirectionX="left" anchorAlignY="bottom">
      <SecretsList definitionId="Mocks" />
    </Popover>
    <UnsetButton
      v-if="!disabled"
      :disabled="disableUnset"
      @click="unsetField"
    />
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
import UnsetButton from "./UnsetButton.vue";

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
