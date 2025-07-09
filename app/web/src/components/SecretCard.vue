<template>
  <div
    :class="
      clsx(
        'flex items-center px-xs py-[1px] cursor-pointer border group/secretcard',
        variant === 'minimal'
          ? [
              'flex-row border-transparent',
              themeClasses(
                'hover:border-action-500 hover:text-action-500 text-shade-100',
                'hover:border-action-300 hover:text-action-300 text-shade-0',
              ),
            ]
          : [
              'flex-col rounded',
              themeClasses('border-neutral-400', 'border-neutral-600'),
              isUsable
                ? themeClasses(
                    'hover:border-action-500 hover:text-action-500 text-shade-100',
                    'hover:border-action-300 hover:text-action-300 text-shade-0',
                  )
                : themeClasses(
                    'hover:border-destructive-600 text-shade-100 hover:text-destructive-600 bg-caution-lines-light',
                    'hover:border-destructive-500 text-shade-0 hover:text-destructive-500 bg-caution-lines-dark',
                  ),
            ],
        selected &&
          themeClasses(
            'bg-action-100 border-action-500',
            'bg-action-900  border-action-300',
          ),
      )
    "
    @click="emit('select')"
  >
    <!-- Minimal view, used in SecretsPanel -->
    <template v-if="variant === 'minimal'">
      <div class="flex-grow text-xs truncate">
        {{ secret.name }}
      </div>
      <IconButton
        v-if="variant === 'minimal'"
        size="sm"
        class="flex-none"
        icon="dots-vertical"
        iconIdleTone="neutral"
        :selected="menuRef?.isOpen"
        @click="onClick"
      />
      <DropdownMenu ref="menuRef">
        <DropdownMenuItem
          icon="cursor"
          label="Replace Secret"
          @select="emit('edit')"
        />
        <DropdownMenuItem
          :disabled="secret.connectedComponents.length > 0"
          icon="trash"
          label="Delete"
          @select="deleteSecret"
        />
      </DropdownMenu>
    </template>

    <!-- Detailed view, used in SecretsModal -->
    <template v-else>
      <TruncateWithTooltip class="w-full text-sm font-bold">
        {{ secret.name }}
      </TruncateWithTooltip>

      <div
        v-if="!isUsable"
        :class="
          clsx(
            'w-full text-xs font-bold',
            themeClasses('text-destructive-600', 'text-destructive-500'),
          )
        "
      >
        Created in another workspace, cannot use this secret.
      </div>
      <div v-else-if="secret.updatedInfo" :class="detailClasses">
        Updated:
        <span class="italic">
          <Timestamp
            :date="new Date(secret.updatedInfo.timestamp)"
            relative
            size="normal"
          />
          by
          {{ secret.updatedInfo.actor.label }}
        </span>
      </div>
      <div v-else :class="detailClasses">
        Created:
        <span class="italic">
          <Timestamp
            :date="new Date(secret.createdInfo.timestamp)"
            relative
            size="normal"
          />
          by
          {{ secret.createdInfo.actor.label }}
        </span>
      </div>

      <div v-if="secret.connectedComponents.length > 0" :class="detailClasses">
        Connected Components: {{ secret.connectedComponents.length }}
      </div>

      <div :class="detailClasses" class="line-clamp-2">
        <template v-if="secret.description">
          Description: <span class="italic">{{ secret.description }}</span>
        </template>
        <template v-else> No Description Found </template>
      </div>
    </template>
  </div>
</template>

<script lang="ts" setup>
import {
  DropdownMenu,
  DropdownMenuItem,
  themeClasses,
  Timestamp,
  IconButton,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import { computed, PropType, ref } from "vue";
import clsx from "clsx";
import { Secret, useSecretsStore } from "@/store/secrets.store";

// The "minimal" variant is used in the SecretsPanel and the "detailed" variant is used in the SecretsModal
export type SecretCardVariant = "minimal" | "detailed";

const props = defineProps({
  secret: { type: Object as PropType<Secret>, required: true },
  selected: { type: Boolean },
  variant: { type: String as PropType<SecretCardVariant>, default: "minimal" },
});

const isUsable = computed(() => props.secret.isUsable);

const secretsStore = useSecretsStore();
const menuRef = ref<InstanceType<typeof DropdownMenu>>();

const onClick = (e: MouseEvent) => {
  menuRef.value?.open(e);
};

const deleteSecret = async () => {
  if (!props.secret || !props.secret.id) return;
  emit("deleted");
  await secretsStore.DELETE_SECRET(props.secret.id);
};

const detailClasses = computed(() =>
  clsx(
    "w-full text-xs",
    isUsable.value
      ? themeClasses(
          "text-neutral-500 group-hover/secretcard:text-action-500",
          "text-neutral-400 group-hover/secretcard:text-action-300",
        )
      : themeClasses(
          "text-neutral-500 group-hover/secretcard:text-destructive-600",
          "text-neutral-400 group-hover/secretcard:text-destructive-500",
        ),
  ),
);

const emit = defineEmits<{
  (e: "select"): void;
  (e: "edit"): void;
  (e: "deleted"): void;
}>();
</script>
