<template>
  <Modal ref="modalRef" title="Controls" size="xl">
    <div
      :class="
        clsx(
          'flex flex-col gap-xs scrollable max-h-[70vh]',
          '[&>div]:flex [&>*]:flex-row [&>*]:items-center [&>*]:gap-xs',
          '[&_.keys]:w-12 [&_.keys]:flex [&_.keys]:flex-row [&_.keys]:items-center [&_.keys]:justify-center',
          '[&_.key]:text-xl [&_.key]:font-bold [&_.key]:leading-none [&_.key]:flex-grow [&_.key]:text-center',
        )
      "
    >
      <span class="font-bold underline">General</span>
      <div>
        <div class="keys">
          <div class="key">?</div>
        </div>
        <div>Press <TextPill tighter variant="key">?</TextPill> to open this modal.</div>
      </div>
      <div>
        <div class="keys">
          <div class="key">M</div>
        </div>
        <div>
          Press <TextPill tighter variant="key">M</TextPill> when on the map view to toggle the mini-map open or closed.
        </div>
      </div>
      <div>
        <div class="keys">
          <div class="key">K</div>
        </div>
        <div>Press <TextPill tighter variant="key">K</TextPill> to select the search bar.</div>
      </div>
      <div>
        <div class="keys">
          <div class="key">R</div>
        </div>
        <div>Press <TextPill tighter variant="key">R</TextPill> to open the review screen.</div>
      </div>
      <div>
        <div class="keys">
          <Icon name="mouse-left-click-outline" />
        </div>
        <div>Left click on a component to see details about it.</div>
      </div>
      <div>
        <div class="keys">
          <Icon name="mouse-right-click-outline" />
        </div>
        <div>Right click on a component to take action.</div>
      </div>

      <span class="font-bold mt-md underline">Selection</span>
      <div>
        <div class="keys">
          <div class="key">Tab</div>
        </div>
        <div>
          Press <TextPill tighter variant="key">Tab</TextPill> and
          <TextPill tighter variant="key">Shift</TextPill>+<TextPill tighter variant="key">Tab</TextPill>
          to select components on the grid.
        </div>
      </div>
      <div>
        <div class="keys">
          <Icon name="command" />
          <div class="key">A</div>
        </div>
        <div>
          Press <TextPill tighter variant="key">⌘</TextPill>+<TextPill tighter variant="key">A</TextPill>
          to select all components in the grid.
        </div>
      </div>
      <div>
        <div class="keys">
          <Icon name="esc" />
        </div>
        <div>Press <TextPill tighter variant="key">Escape</TextPill> to clear your current selection.</div>
      </div>
      <div>
        <div class="keys">
          <Icon name="spacebar" />
        </div>
        <div>
          Press <TextPill tighter variant="key">Enter</TextPill> or
          <TextPill tighter variant="key">Spacebar</TextPill> to open a selected component's details.
        </div>
      </div>

      <span class="font-bold mt-md underline">Actions</span>
      <div>
        <div class="keys">
          <div class="key">C</div>
        </div>
        <div>Press <TextPill tighter variant="key">C</TextPill> to create a new change set.</div>
      </div>
      <div>
        <div class="keys">
          <div class="key">N</div>
        </div>
        <div>Press <TextPill tighter variant="key">N</TextPill> to add a new component.</div>
      </div>

      <span class="font-bold mt-md underline">Selected Component Actions</span>
      <div>
        <div class="keys">
          <div class="key">D</div>
        </div>
        <div>
          Press <TextPill tighter variant="key">D</TextPill>
          to duplicate the selected component.
        </div>
      </div>
      <div>
        <div class="keys">
          <div class="key">E</div>
        </div>
        <div>Press <TextPill tighter variant="key">E</TextPill> to erase the selected component.</div>
      </div>
      <div>
        <div class="keys">
          <div class="key">P</div>
        </div>
        <div>Press <TextPill tighter variant="key">P</TextPill> to pin or unpin the selected component.</div>
      </div>
      <div>
        <div class="keys">
          <div class="key">F</div>
        </div>
        <div>
          Press <TextPill tighter variant="key">F</TextPill> to restore the selected component (if set for deletion).
        </div>
      </div>
      <div>
        <div class="keys">
          <div class="key">U</div>
        </div>
        <div>
          Press <TextPill tighter variant="key">U</TextPill> to upgrade the selected component (if upgradeable).
        </div>
      </div>
      <div>
        <div class="keys">
          <Icon name="backspace" />
        </div>
        <div>
          Press <TextPill tighter variant="key">Backspace</TextPill> or
          <TextPill tighter variant="key">Delete</TextPill> to delete the selected component.
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
