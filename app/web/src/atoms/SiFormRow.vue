<template>
  <div class="flex flex-row items-center w-full pb-2">
    <div class="w-1/2 pr-2 text-sm text-right text-gray-400 align-middle">
      <slot name="label"></slot>
    </div>
    <div class="w-1/2 align-middle">
      <slot name="widget"></slot>
    </div>
  </div>
</template>
