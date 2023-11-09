<template>
  <div v-show="isShown" class="flex content-center mb-3">
    <div class="flex grow">
      <div class="w-full h-44">
        <label v-if="id" :for="id" class="block text-sm font-medium mb-1">
          {{ props.name }}
        </label>

        <CodeEditor
          v-model="currentValue"
          disabled
          class="cursor-pointer"
          @click="editorModalRef?.open()"
        />

        <div
          v-if="inError"
          class="absolute inset-y-0 right-0 pr-2 flex items-center text-destructive-400"
        >
          <Icon name="exclamation-circle" />
        </div>

        <p v-if="docLink" class="mt-2 text-xs text-action-500">
          <a :href="docLink" target="_blank" class="hover:underline">
            Documentation
          </a>
        </p>

        <SiValidation
          :value="currentValue"
          :validations="validations"
          class="mt-2"
          @errors="setInError($event)"
        />
      </div>
    </div>

    <UnsetButton
      v-if="canUnset && !disabled"
      :disabled="disableUnset"
      @click="unsetField"
    />

    <Modal ref="editorModalRef" size="6xl">
      <CodeEditor
        :id="id"
        v-model="currentValue"
        :disabled="disabled"
        class="height-widget-code-editor"
        @blur="blur"
      />
    </Modal>
  </div>
</template>

<script setup lang="ts">
import { ref, toRefs, computed, watch, onBeforeUnmount } from "vue";
import * as _ from "lodash-es";
import { Icon, Modal } from "@si/vue-lib/design-system";
import CodeEditor from "@/components/CodeEditor.vue";
import { usePropertyEditorIsShown } from "@/utils/usePropertyEditorIsShown";
import SiValidation from "@/components/SiValidation.vue";
import {
  PropertyEditorValidation,
  UpdatedProperty,
  PropertyPath,
  PropertyEditorPropKind,
} from "@/api/sdf/dal/property_editor";
import {
  useValidations,
  usePropertyEditorValidations,
} from "@/utils/input_validations";
import { useComponentsStore } from "@/store/components.store";
import { useAuthStore } from "@/store/auth.store";
import UnsetButton from "./UnsetButton.vue";

const componentsStore = useComponentsStore();
const authStore = useAuthStore();

const id = computed(() =>
  componentsStore.selectedComponentId
    ? `value-${componentsStore.selectedComponentId}-${props.propId}-${authStore.user?.pk}`
    : undefined,
);

const props = defineProps<{
  name: string;
  path?: PropertyPath;
  collapsedPaths: Array<Array<string>>;
  value: unknown;
  propId: string;
  valueId: string;
  propKind: PropertyEditorPropKind;
  docLink?: string;
  validation?: PropertyEditorValidation;
  disabled?: boolean;
}>();

const editorModalRef = ref();

const blur = () => {
  setDirty();
  setField();
};
const emit = defineEmits<{
  (e: "updatedProperty", v: UpdatedProperty): void;
  (e: "error", v: boolean): void;
}>();

const { name, path, collapsedPaths, valueId, propId, value, validation } =
  toRefs(props);

const currentValue = ref<string>("");
watch(
  value,
  (newValue, oldValue) => {
    if (oldValue !== newValue) {
      if (_.isString(newValue)) {
        currentValue.value = newValue;
      } else if (_.isNumber(newValue)) {
        currentValue.value = `${newValue}`;
      } else {
        currentValue.value = "";
      }
    }
  },
  { immediate: true },
);

const disableUnset = computed(() => {
  if (_.isNull(value.value)) {
    return true;
  } else {
    return false;
  }
});

const { isShown } = usePropertyEditorIsShown(name, collapsedPaths, path);

const setField = () => {
  if (
    !_.isNull(currentValue.value) &&
    currentValue.value !== props.value &&
    ((!props.value && currentValue.value) || props.value)
  ) {
    if (props.propKind === "integer") {
      emit("updatedProperty", {
        value: _.parseInt(currentValue.value, 10),
        propId: propId.value,
        valueId: valueId.value,
      });
    } else {
      emit("updatedProperty", {
        value: currentValue.value,
        propId: propId.value,
        valueId: valueId.value,
      });
    }
  }
};

const unsetField = () => {
  emit("updatedProperty", {
    value: null,
    propId: propId.value,
    valueId: valueId.value,
  });
};

const validations = usePropertyEditorValidations(validation);

const alwaysValidate = ref(true);
const { inError, setInError, setDirty } = useValidations(
  alwaysValidate,
  setField,
  (inError: boolean) => emit("error", inError),
);

onBeforeUnmount(setField);

const canUnset = computed(() => {
  return (props.path?.triggerPath ?? []).join(".") !== "name.si.root";
});
</script>

<style lang="less">
.height-widget-code-editor {
  height: 80vh;
}
</style>
