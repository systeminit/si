<template>
  <Teleport to=".si-panel-right">
    <Transition
      enterActiveClass="duration-100 ease-out"
      enterFromClass="translate-x-[500px]"
      leaveActiveClass="duration-100 ease-in"
      leaveToClass="translate-x-[500px]"
    >
      <div v-if="open" class="absolute w-[500px] h-full left-[-500px] bg-white dark:bg-neutral-800 z-[-10]">
        <slot />
      </div>
    </Transition>
  </Teleport>
</template>

<script lang="ts" setup>
defineProps({
  open: { type: Boolean },
});
</script>
