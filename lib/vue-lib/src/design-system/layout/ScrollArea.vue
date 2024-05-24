<template>
  <div class="flex flex-col absolute inset-0">
    <div v-if="$slots.top" class="shrink-0">
      <slot name="top" />
    </div>
    <div class="overflow-auto flex-grow relative scroll-slot">
      <slot />
    </div>
    <div v-if="$slots.bottom" class="shrink-0">
      <slot name="bottom" />
    </div>
  </div>
</template>
