<template>
  <Modal ref="modalRef" title="Diagram Controls" size="xl">
    <div class="flex flex-col gap-xs">
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <Icon name="mouse-and-touchpad" />
        </div>
        <div>Use the mouse to add, create, and select Components.</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <Icon name="plus-minus" />
        </div>
        <div>Use + and - to adjust the zoom level.</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <Icon name="command" />
          <Icon name="mouse-scroll" />
        </div>
        <div>
          Hold ⌘ to zoom using the scroll wheel, or just pinch on trackpad.
        </div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <Icon name="option" />
          <Icon name="click" />
        </div>
        <div>Hold ⌥ and click to force drag to select.</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <Icon name="option" />
          <Icon name="shift" />
        </div>
        <div>Press ⌥ and Shift to close/open both side panels.</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <Icon name="arrows-out" />
        </div>
        <div>
          Use the arrow keys to nudge selection, hold shift to nudge further.
        </div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <Icon name="spacebar" />
        </div>
        <div>Hold spacebar to activate drag-to-scroll.</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <Icon name="esc" />
        </div>
        <div>Press escape to clear selection.</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <div class="text-xl font-bold leading-none flex-grow text-center">
            R
          </div>
        </div>
        <div>Press R to refresh the resource for a component or frame.</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <div class="text-xl font-bold leading-none flex-grow text-center">
            N
          </div>
        </div>
        <div>Press N to rename a component or frame.</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <Icon name="command" />
          <div class="text-xl font-bold leading-none flex-grow text-center">
            C
          </div>
        </div>
        <div>Press ⌘ and C to copy selection.</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <Icon name="backspace" />
        </div>
        <div>Press backspace or delete to delete selection.</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <Icon name="command" />
          <div class="text-xl font-bold leading-none flex-grow text-center">
            E
          </div>
        </div>
        <div>Press ⌘ and E to erase selection.</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <div class="text-xl font-bold leading-none flex-grow text-center">
            \
          </div>
        </div>
        <div>Press the backslash key to collapse selected frame(s).</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <Icon name="command" />
          <div class="text-xl font-bold leading-none flex-grow text-center">
            \
          </div>
        </div>
        <div>Press ⌘ and \ to collapse all frames.</div>
      </div>
    </div>
  </Modal>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import { Modal, useModal, Icon } from "@si/vue-lib/design-system";

const modalRef = ref<InstanceType<typeof Modal>>();
const { open, close } = useModal(modalRef);
defineExpose({ open, close });
</script>
