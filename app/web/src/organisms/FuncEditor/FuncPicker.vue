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
</template>

<script lang="ts" setup>
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import SiFuncSprite from "@/molecules/SiFuncSprite.vue";
import { ListedFuncView, ListFuncsResponse } from "@/service/func/list_funcs";

defineProps<{
  funcList: ListFuncsResponse;
  selectedFuncId: number;
}>();

const emits = defineEmits<{
  (e: "selectedFunc", v: ListedFuncView): void;
}>();

const selectFunc = (func: ListedFuncView) => {
  emits("selectedFunc", func);
};
</script>
