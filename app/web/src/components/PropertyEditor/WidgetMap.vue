<template>
  <div>
    <WidgetHeader
      :collapsed-paths="props.collapsedPaths"
      :name="props.name"
      :path="props.path"
      @toggle-collapsed="setCollapsed($event)"
    />
    <div
      v-show="isShown && !isCollapsed && !props.disabled"
      class="pl-8 flex flex-col w-full pt-4"
    >
      <div class="w-full pr-24">
        <SiTextBox :id="newKeyId" v-model="newKey" title="key" />
      </div>
      <div class="flex pt-4 pr-16">
        <VButton
          :disabled="submitDisabled"
          icon="plus"
          size="xs"
          tone="neutral"
          label="Add to map"
          @click="addToMap()"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, toRefs } from "vue";
import { VButton } from "@si/vue-lib/design-system";
import SiTextBox from "@/components/SiTextBox.vue";
import { usePropertyEditorIsShown } from "@/utils/usePropertyEditorIsShown";
import { AddToMap, PropertyPath } from "@/api/sdf/dal/property_editor";
import WidgetHeader from "./WidgetHeader.vue";

const props = defineProps<{
  name: string;
  path?: PropertyPath;
  collapsedPaths: Array<Array<string>>;
  disabled?: boolean;
  propId: string;
  valueId: string;
  arrayLength?: number;
}>();
const emits = defineEmits<{
  (e: "toggle-collapsed", path: Array<string>): void;
  (e: "addToMap", v: AddToMap): void;
}>();

const setCollapsed = (path: Array<string>) => {
  emits("toggle-collapsed", path);
};

const newKeyId = ref<string>(`newMap${props.valueId}`);
const newKey = ref<string>("");
const submitDisabled = computed(() => {
  return !newKey.value;
});

const { name, path, collapsedPaths } = toRefs(props);
const { isShown, isCollapsed } = usePropertyEditorIsShown(
  name,
  collapsedPaths,
  path,
  true,
);

const addToMap = () => {
  emits("addToMap", {
    propId: props.propId,
    valueId: props.valueId,
    key: newKey.value,
  });
  newKey.value = "";
};
</script>
