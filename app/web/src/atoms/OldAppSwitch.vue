<template>
  <button data-test="home" @click="onHome">
    <SiIcon tooltip-text="Home">
      <SwitchHorizontalIcon :class="buttonClass" />
    </SiIcon>
  </button>
</template>

<script setup lang="ts">
import { useRouter } from "vue-router";
import { SwitchHorizontalIcon } from "@heroicons/vue/outline";
import SiIcon from "@/atoms/SiIcon.vue";
import { computed } from "vue";

const props = defineProps<{
  buttonClass?: string;
}>();

const buttonClass = computed((): string => {
  if (props.buttonClass) {
    return props.buttonClass;
  }
  return "text-black";
});

const router = useRouter();
const onHome = async () => {
  await router.push({ name: "home" });
};
</script>
