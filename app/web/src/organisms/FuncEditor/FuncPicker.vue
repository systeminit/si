<template>
  <ul class="overflow-y-auto">
    <SiCollapsible label="Qualification Functions" as="li" content-as="ul">
      <li v-for="func in funcList.qualifications" :key="func.id">
        <SiFuncSprite
          :name="func.name"
          color="#921ed6"
          :class="selectedFuncId == func.id ? 'bg-action-500' : ''"
          class="border-b-2 dark:border-neutral-600 hover:bg-action-500 dark:text-white hover:text-white hover:cursor-pointer"
          @click="selectFunc(func)"
        />
      </li>
    </SiCollapsible>
  </ul>
  <div
    class="absolute bottom-0 w-full h-12 text-right p-2 border-t border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800"
  >
    <SiButton
      icon="plus"
      kind="save"
      label="Create Function"
      size="lg"
      @click="createFunc"
    />
  </div>
</template>

<script lang="ts" setup>
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import SiFuncSprite from "@/molecules/SiFuncSprite.vue";
import SiButton from "@/atoms/SiButton.vue";
import { ListedFuncView, ListFuncsResponse } from "@/service/func/list_funcs";

defineProps<{
  funcList: ListFuncsResponse;
  selectedFuncId: number;
}>();

const emits = defineEmits<{
  (e: "selectedFunc", v: ListedFuncView): void;
  (e: "createFunc"): void;
}>();

const selectFunc = (func: ListedFuncView) => {
  emits("selectedFunc", func);
};

const createFunc = () => {
  emits("createFunc");
};
</script>
