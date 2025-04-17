<template>
  <Transition name="float-fade">
    <div v-if="isOpen" class="fixed inset-0 z-50" @mousedown.self="close">
      <div
        ref="panelRef"
        :class="[
          'absolute rounded-lg shadow-lg z-50 overflow-hidden',
          'bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-600',
          position === 'top-left'
            ? 'top-24 left-12'
            : position === 'top-right'
            ? 'top-24 right-12'
            : position === 'bottom-left'
            ? 'bottom-12 left-12'
            : 'bottom-12 right-12',
        ]"
        :style="{
          width: `${width}px`,
          height: `${height}px`,
          maxWidth: 'calc(100vw - 100px)',
          maxHeight: 'calc(100vh - 100px)',
        }"
      >
        <!-- Header -->
        <div
          class="flex items-center justify-between px-4 py-2 border-b dark:border-neutral-600 cursor-move"
          @mousedown="startDrag"
        >
          <h3 class="font-medium text-sm">{{ title }}</h3>
          <button
            class="p-1 rounded-full hover:bg-neutral-100 dark:hover:bg-neutral-700"
            @click="close"
          >
            <Icon name="x" size="sm" />
          </button>
        </div>

        <!-- Content -->
        <div class="h-[calc(100%-40px)] overflow-auto">
          <slot></slot>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script lang="ts" setup>
import { ref, onMounted, onUnmounted } from "vue";
import { Icon } from "@si/vue-lib/design-system";

const props = defineProps<{
  title: string;
  isOpen: boolean;
  width?: number;
  height?: number;
  position?: "top-left" | "top-right" | "bottom-left" | "bottom-right";
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "position-change", x: number, y: number): void;
}>();

// Default values
const width = props.width || 320;
const height = props.height || 500;
const position = props.position || "top-left";

// Panel ref for drag handling
const panelRef = ref<HTMLElement | null>(null);

// Dragging state
const isDragging = ref(false);
const dragOffset = ref({ x: 0, y: 0 });

function close() {
  emit("close");
}

function startDrag(e: MouseEvent) {
  if (!panelRef.value) return;

  isDragging.value = true;
  const rect = panelRef.value.getBoundingClientRect();
  dragOffset.value = {
    x: e.clientX - rect.left,
    y: e.clientY - rect.top,
  };

  document.addEventListener("mousemove", handleDrag);
  document.addEventListener("mouseup", stopDrag);
}

function handleDrag(e: MouseEvent) {
  if (!isDragging.value || !panelRef.value) return;

  const x = e.clientX - dragOffset.value.x;
  const y = e.clientY - dragOffset.value.y;

  // Keep panel within viewport
  const maxX = window.innerWidth - panelRef.value.offsetWidth;
  const maxY = window.innerHeight - panelRef.value.offsetHeight;

  const boundedX = Math.max(0, Math.min(x, maxX));
  const boundedY = Math.max(0, Math.min(y, maxY));

  panelRef.value.style.left = `${boundedX}px`;
  panelRef.value.style.top = `${boundedY}px`;

  // Remove any right/bottom positioning if it was set
  panelRef.value.style.right = "auto";
  panelRef.value.style.bottom = "auto";

  emit("position-change", boundedX, boundedY);
}

function stopDrag() {
  isDragging.value = false;
  document.removeEventListener("mousemove", handleDrag);
  document.removeEventListener("mouseup", stopDrag);
}

// Clean up event listeners on component unmount
onUnmounted(() => {
  document.removeEventListener("mousemove", handleDrag);
  document.removeEventListener("mouseup", stopDrag);
});
</script>

<style scoped>
.float-fade-enter-active,
.float-fade-leave-active {
  transition: opacity 0.2s;
}

.float-fade-enter-from,
.float-fade-leave-to {
  opacity: 0;
}
</style>
