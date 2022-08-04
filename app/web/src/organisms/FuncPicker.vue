<template>
  <ul class="overflow-y-auto">
    <SiCollapsible label="Qualification Functions" as="li" content-as="ul">
      <li v-for="func in funcList.qualifications" :key="func.id">
        <SiNodeSprite
          :name="func.name"
          color="#921ed6"
          class="border-b-2 dark:border-neutral-600 hover:bg-action-500 dark:text-white hover:text-white hover:cursor-pointer"
        />
      </li>
    </SiCollapsible>
  </ul>
</template>

<script lang="ts" setup>
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import SiNodeSprite from "@/molecules/SiNodeSprite.vue";
import { ListFuncsResponse } from "@/service/func/list_funcs";

defineProps<{
  funcList: ListFuncsResponse;
}>();
</script>
