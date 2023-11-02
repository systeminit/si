<template>
  <div
    v-tooltip="tooltip"
    class="h-8 w-8 border-2 rounded-full cursor-pointer flex-none bg-shade-100"
    :style="`border-color: ${color}`"
  >
    <!-- TODO(Wendy) - This should check for and pull the image of the user in question, not the current user's image! -->
    <img
      v-if="user.pictureUrl"
      class="rounded-full bg-white"
      :src="user.pictureUrl"
      referrerpolicy="no-referrer"
    />
    <Icon v-else name="user-circle" size="full" />
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
import { PropType, computed } from "vue";
import { UserInfo } from "./Collaborators.vue";

const props = defineProps({
  tooltip: { type: Object },
  user: { type: Object as PropType<UserInfo>, required: true },
});

const color = computed(() => {
  if (props.user.color) return props.user.color;
  else return "black";
});
</script>
