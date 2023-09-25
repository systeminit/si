<template>
  <div
    class="flex flex-row w-full text-neutral-400 border-b dark:border-neutral-500 items-center"
  >
    <div class="grow uppercase text-md pl-3 py-5 font-bold"><slot /></div>
    <div class="flex-none pr-3"><slot name="icon" /></div>
  </div>
</template>

<script setup lang="ts"></script>
