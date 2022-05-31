<template>
  <div class="block">
    <SiButtonIcon
      v-if="modelValue"
      tooltip-text="Lock Selection"
      @click="toggle()"
    >
      <LockClosedIcon class="text-yellow-300" />
    </SiButtonIcon>
    <SiButtonIcon v-else tooltip-text="Unlock Selection" @click="toggle()">
      <LockOpenIcon class="text-yellow-300" />
    </SiButtonIcon>
  </div>

  <!--
  <button class="pl-4 text-sm focus:outline-none" @click="toggle()">
    <VueFeather v-if="modelValue" size="1rem" type="lock" class="locked" />
    <VueFeather v-else size="1rem" type="unlock" class="unlocked" />
  </button>
    -->
</template>

<script setup lang="ts">
import SiButtonIcon from "./SiButtonIcon.vue";
import { LockClosedIcon, LockOpenIcon } from "@heroicons/vue/solid";

const props = defineProps({
  modelValue: { type: Boolean, required: true },
});

const emits = defineEmits(["update:modelValue"]);

const toggle = () => {
  emits("update:modelValue", !props.modelValue);
};
</script>

<style scoped>
.unlocked {
  color: #c6c6c6;
}

.locked {
  color: #e3ddba;
}
</style>
