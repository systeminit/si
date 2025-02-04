<template>
  <div class="h-7">
    <DropdownMenuButton
      v-if="selectUsers.length > 0"
      ref="emailInputRef"
      v-model="userId"
      :options="selectUsers"
      class="w-full"
      checkable
      placeholder="Select user..."
      minWidthToAnchor
      alignRightOnAnchor
      @select="setUser"
    />
    <div
      v-else
      :class="
        clsx(
          'flex flex-row items-center p-2xs mb-[-1px] h-7',
          'font-mono text-[13px] text-left truncate relative border',
          'cursor-not-allowed',
          themeClasses(
            'text-neutral-500 border-neutral-400 bg-caution-lines-light',
            'text-neutral-400 border-neutral-600 bg-caution-lines-dark',
          ),
        )
      "
    >
      {{ noUsersLabel }}
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, computed, onBeforeMount } from "vue";
import { DropdownMenuButton, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useAuthStore, UserId } from "@/store/auth.store";
import { useChangeSetsStore } from "@/store/change_sets.store";

const authStore = useAuthStore();
const changeSetsStore = useChangeSetsStore();

const props = defineProps({
  usersToFilterOut: { type: Array<UserId>, default: [] },
  noUsersLabel: { type: String, default: "No available users to select" },
});

const userId = ref("");
const setUser = (id: string) => {
  emit("select", id);
};

onBeforeMount(() => {
  if (changeSetsStore.selectedWorkspacePk) {
    authStore.LIST_WORKSPACE_USERS(changeSetsStore.selectedWorkspacePk);
  }
});

const rawUsers = computed(() => authStore.workspaceUsers);
const selectUsers = computed(() => {
  if (!rawUsers.value) return [];
  else
    return Object.values(rawUsers.value)
      .filter((user) => !props.usersToFilterOut.includes(user.id))
      .map((user) => ({
        value: user.id,
        label: `${user.name} (${user.email})`,
      }));
});

const emit = defineEmits<{
  (e: "select", userId: string): void;
}>();

const clearSelection = () => {
  userId.value = "";
};

defineExpose({ userId, clearSelection });
</script>
