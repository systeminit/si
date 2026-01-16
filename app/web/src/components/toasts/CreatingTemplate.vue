<template>
  <div :class="clsx('container flex flex-row rounded', updating ? 'cursor-progress' : 'cursor-pointer')">
    <div class="flex flex-row gap-xs px-sm py-xs items-center">
      <template v-if="updating">
        <Icon name="loader" size="xl" tone="action" />
        <p class="text-sm">Creating Template...</p>
      </template>
      <template v-else>
        <Icon name="check-hex-outline" size="xl" tone="success" />
        <div class="text-sm">
          Template
          <span class="font-bold">{{ templateName ? `&quot;${templateName}&quot; ` : "" }}</span
          >has been created successfully!
        </div>
      </template>
    </div>
    <div
      v-if="!updating"
      :class="
        clsx(
          'flex flex-col text-sm items-stretch text-center justify-center border-l',
          themeClasses('border-neutral-200 text-action-500', 'border-neutral-600 text-action-300'),
        )
      "
    >
      <button
        :class="
          clsx(
            'p-xs border-b hover:underline',
            themeClasses('border-neutral-200', 'border-neutral-600 focus:text-shade-0 focus:bg-action-300'),
          )
        "
        @click="goToFunc"
      >
        View Code
      </button>
      <button :class="clsx('p-xs hover:underline', themeClasses('', 'focus:text-shade-0 focus:bg-action-300'))">
        Close
      </button>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";

const props = defineProps({
  updating: { type: Boolean },
  templateName: { type: String },
  schemaVariantId: { type: String },
  funcId: { type: String },
  router: { type: Function },
});

const goToFunc = () => {
  const s = `a_${props.schemaVariantId}|f_${props.funcId}`;
  if (props.router) {
    props.router(s);
  }
};
</script>
