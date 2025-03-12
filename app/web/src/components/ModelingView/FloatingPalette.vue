<template>
<div
  id="floating-palette"
  :class="
    clsx(
      'fixed flex flex-col text-sm p-sm',
      'outline outline-offset-0  outline-gray-900 outline-0 dark:outline-1',
      themeClasses(
          'bg-shade-0 border-neutral-300',
          'bg-neutral-700 border-neutral-800',
        ),
        'w-[230px] h-[150px] z-[35] border-r-2 border-b-2',
    )
  "
  :style="{ left: `${left}px`, top: `${top}px` }"
>
  I am floating!
</div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { themeClasses } from "@si/vue-lib/design-system";
import { ref } from "vue";

const left = ref<number>(400);
const top = ref<number>(400);

</script>