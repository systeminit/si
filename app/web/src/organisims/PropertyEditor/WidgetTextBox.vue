<template>
  <div v-show="isShown">
    <SiTextBox
      v-model="fieldValue"
      :title="props.name"
      :id="fieldId"
      @blur="blurCallback"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, toRefs, computed } from "vue";
import SiTextBox from "@/atoms/SiTextBox2.vue";
import { usePropertyEditorIsShown } from "@/composables/usePropertyEditorIsShown";
import { UpdatedProperty } from "@/api/sdf/dal/property_editor";
import _ from "lodash";

const props = defineProps<{
  name: string;
  path: string[];
  collapsedPaths: Array<Array<string>>;
  value: unknown;
  propId: number;
  valueId: number;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", v: string): void;
  (e: "updatedProperty", v: UpdatedProperty): void;
}>();

const { name, path, collapsedPaths, valueId, propId } = toRefs(props);
const currentValue = ref<string>(String(props.value));

const fieldValue = computed<string>({
  get(): string {
    if (_.isString(props.value)) {
      return props.value;
    } else {
      return "";
    }
  },
  set(value) {
    currentValue.value = value;
    emit("update:modelValue", value);
  },
});

const fieldId = ref([props.name, ...props.path].join("."));

const { isShown } = usePropertyEditorIsShown(name, path, collapsedPaths);

const blurCallback = () => {
  console.log("updated property", {
    fv: currentValue.value,
    propId: propId.value,
    valueId: valueId.value,
  });
  emit("updatedProperty", {
    value: currentValue.value,
    propId: propId.value,
    valueId: valueId.value,
  });
};
</script>
