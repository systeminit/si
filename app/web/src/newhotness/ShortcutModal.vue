<template>
  <Modal ref="modalRef" title="Keyboard Controls" size="xl">
    <div class="flex flex-col gap-xs">
      <!-- TODO(Wendy) - this list of shortcuts is not complete, finish it up! -->
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <div class="text-xl font-bold leading-none flex-grow text-center">
            ?
          </div>
        </div>
        <div>Press ? to open this modal.</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <div class="text-xl font-bold leading-none flex-grow text-center">
            K
          </div>
        </div>
        <div>Press K to select the search bar.</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <div class="text-xl font-bold leading-none flex-grow text-center">
            N
          </div>
        </div>
        <div>Press N to add a new component.</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <div class="text-xl font-bold leading-none flex-grow text-center">
            E
          </div>
        </div>
        <div>Press E to erase the selected component.</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <div class="text-xl font-bold leading-none flex-grow text-center">
            U
          </div>
        </div>
        <div>Press U to upgrade the selected component (if upgradeable).</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <Icon name="backspace" />
        </div>
        <div>Press backspace or delete to delete the selected component.</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <div class="text-xl font-bold leading-none flex-grow text-center">
            R
          </div>
        </div>
        <div>
          Press R to restore the selected component (if set for deletion).
        </div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <Icon name="command" />
          <div class="text-xl font-bold leading-none flex-grow text-center">
            D
          </div>
        </div>
        <div>Press ⌘ and D to duplicate the selected component.</div>
      </div>
      <div class="flex flex-row items-center gap-xs">
        <div class="w-12 flex flex-row items-center justify-center">
          <Icon name="esc" />
        </div>
        <div>Press escape to clear your current selection.</div>
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
