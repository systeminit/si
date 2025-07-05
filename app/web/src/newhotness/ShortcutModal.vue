<template>
  <Modal ref="modalRef" title="Keyboard Controls" size="xl">
    <div
      :class="
        clsx(
          'flex flex-col gap-xs',
          '[&>div]:flex [&>*]:flex-row [&>*]:items-center [&>*]:gap-xs',
          '[&_.keys]:w-12 [&_.keys]:flex [&_.keys]:flex-row [&_.keys]:items-center [&_.keys]:justify-center',
          '[&_.key]:text-xl [&_.key]:font-bold [&_.key]:leading-none [&_.key]:flex-grow [&_.key]:text-center',
        )
      "
    >
      <!-- TODO(Wendy) - this list of shortcuts is not complete, finish it up! -->
      <div>
        <div class="keys">
          <div class="key">?</div>
        </div>
        <div>
          Press <TextPill tighter variant="key">?</TextPill> to open this modal.
        </div>
      </div>
      <div>
        <div class="keys">
          <div class="key">Tab</div>
        </div>
        <div>
          Press <TextPill tighter variant="key">Tab</TextPill> and
          <TextPill tighter variant="key">Shift</TextPill>+<TextPill
            tighter
            variant="key"
            >Tab</TextPill
          >
          to select components on the grid.
        </div>
      </div>
      <div>
        <div class="keys">
          <div class="key">C</div>
        </div>
        <div>
          Press <TextPill tighter variant="key">C</TextPill> to create a new
          change set.
        </div>
      </div>
      <div>
        <div class="keys">
          <div class="key">K</div>
        </div>
        <div>
          Press <TextPill tighter variant="key">K</TextPill> to select the
          search bar.
        </div>
      </div>
      <div>
        <div class="keys">
          <div class="key">N</div>
        </div>
        <div>
          Press <TextPill tighter variant="key">N</TextPill> to add a new
          component.
        </div>
      </div>
      <div>
        <div class="keys">
          <div class="key">E</div>
        </div>
        <div>
          Press <TextPill tighter variant="key">E</TextPill> to erase the
          selected component.
        </div>
      </div>
      <div>
        <div class="keys">
          <div class="key">U</div>
        </div>
        <div>
          Press <TextPill tighter variant="key">U</TextPill> to upgrade the
          selected component (if upgradeable).
        </div>
      </div>
      <div>
        <div class="keys">
          <div class="key">P</div>
        </div>
        <div>
          Press <TextPill tighter variant="key">P</TextPill> to pin or unpin the
          selected component.
        </div>
      </div>
      <div>
        <div class="keys">
          <Icon name="backspace" />
        </div>
        <div>
          Press <TextPill tighter variant="key">Backspace</TextPill> or
          <TextPill tighter variant="key">Delete</TextPill> to delete the
          selected component.
        </div>
      </div>
      <div>
        <div class="keys">
          <div class="key">R</div>
        </div>
        <div>
          Press <TextPill tighter variant="key">R</TextPill> to restore the
          selected component (if set for deletion).
        </div>
      </div>
      <div>
        <div class="keys">
          <Icon name="command" />
          <div class="key">D</div>
        </div>
        <div>
          Press <TextPill tighter variant="key">⌘</TextPill>+<TextPill
            tighter
            variant="key"
            >D</TextPill
          >
          to duplicate the selected component.
        </div>
      </div>
      <div>
        <div class="keys">
          <Icon name="command" />
          <div class="key">A</div>
        </div>
        <div>
          Press <TextPill tighter variant="key">⌘</TextPill>+<TextPill
            tighter
            variant="key"
            >A</TextPill
          >
          to select all components in the grid.
        </div>
      </div>
      <div>
        <div class="keys">
          <Icon name="esc" />
        </div>
        <div>
          Press <TextPill tighter variant="key">Escape</TextPill> to clear your
          current selection.
        </div>
      </div>
      <div>
        <div class="keys">
          <div class="key">E</div>
        </div>
        <div>
          Press <TextPill tighter variant="key">E</TextPill>
          to erase the selected component.
        </div>
      </div>
    </div>
  </Modal>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import { Modal, useModal, Icon, TextPill } from "@si/vue-lib/design-system";
import clsx from "clsx";

const modalRef = ref<InstanceType<typeof Modal>>();
const { open, close } = useModal(modalRef);
defineExpose({ open, close, isOpen: modalRef.value?.isOpen });
</script>
