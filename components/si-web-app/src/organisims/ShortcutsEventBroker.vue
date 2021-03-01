<template>
  <div>
    <slot />
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { PanelEventBus } from "@/atoms/PanelEventBus";
import _ from "lodash";

// @keydown.space="handleSpacebar($event)"

interface KeyboardKey {
  long: string;
  short: string;
}

const spacebarKey: KeyboardKey = {
  long: "Spacebar",
  short: " ",
};

type KeyboardEventDown = "keyDown";
type KeyboardEventUp = "keyUp";
type KeyboardEventKind = KeyboardEventDown | KeyboardEventUp;

export interface ShortcutRegistrationEvent {
  context: ShortcutContext;
}

export interface ShortcutContext {
  id: string;
  isActive: boolean;
}

export enum ShortcutActions {
  StartPan = "pan-start",
  EndPan = "pan-end",
  Maximize = "panel-maximize",
}

export interface ShortcutUpdateEvent {
  action: ShortcutActions;
}

export interface MouseRegistrationEvent {
  context: MouseContext;
}

export interface MouseContext {
  id: string;
  isActive: boolean;
}

interface IData {
  eligibleContexts: Array<string>;
  spacebarIsPressed: boolean;
  mouseIsClicked: boolean;
  isPanning: boolean;
}

export default Vue.extend({
  name: "ShortcutsEventBroker",
  data(): IData {
    return {
      eligibleContexts: [],
      spacebarIsPressed: false,
      mouseIsClicked: false,
      isPanning: false,
    };
  },
  mounted: function() {
    this.registerEvents();
  },
  beforeDestroy() {
    this.deRegisterEvents();
  },
  methods: {
    registerEvents(): void {
      document.addEventListener("keydown", this.handleKeyDown);
      this.$once("hook:beforeDestroy", () => {
        document.removeEventListener("keydown", this.handleKeyDown);
      });
      document.addEventListener("keyup", this.handleKeyUp);
      this.$once("hook:beforeDestroy", () => {
        document.removeEventListener("keyup", this.handleKeyUp);
      });
      PanelEventBus.$on(
        "shortcuts-registration-update",
        this.handleShortcutsRegistrationUpdate,
      );
      PanelEventBus.$on(
        "mouse-registration-update",
        this.handleMouseRegistrationUpdate,
      );
    },
    deRegisterEvents(): void {
      PanelEventBus.$off(
        "shortcuts-registration-update",
        this.handleShortcutsRegistrationUpdate,
      );
      PanelEventBus.$off(
        "mouse-registration-update",
        this.handleMouseRegistrationUpdate,
      );

      document.removeEventListener("keydown", this.handleKeyDown);
      document.removeEventListener("keyup", this.handleKeyUp);
    },
    handleShortcutsRegistrationUpdate(e: ShortcutRegistrationEvent) {
      if (e.context.isActive) {
        this.eligibleContexts.push(e.context.id);
      } else {
        let i = this.eligibleContexts.indexOf(e.context.id);
        this.eligibleContexts = this.eligibleContexts.splice(i, 1);
      }
    },
    handleMouseRegistrationUpdate(e: MouseRegistrationEvent) {
      if (e.context.isActive) {
        this.mouseIsClicked = true;

        if (this.spacebarIsPressed && this.mouseIsClicked) {
          this.isPanning = true;
        }
      } else {
        this.mouseIsClicked = false;
      }
    },
    handleKeyDown(e: KeyboardEvent): void {
      if (e.key === spacebarKey.long || e.key === spacebarKey.short) {
        e.preventDefault();
        this.handleSpacebar("keyDown");
      }
    },
    handleKeyUp(e: KeyboardEvent): void {
      if (e.key === spacebarKey.long || e.key === spacebarKey.short) {
        e.preventDefault();
        this.handleSpacebar("keyUp");
      }
    },
    handleSpacebar(e: KeyboardEventKind) {
      if (e == "keyDown") {
        this.spacebarDown();
      } else if (e == "keyUp") {
        this.spacebarUp();
      }
    },
    spacebarDown(): void {
      this.spacebarIsPressed = true;
      if (this.isPanning) {
        for (var c of this.eligibleContexts) {
          if (_.startsWith(c, "graphViewer")) {
            let e: ShortcutUpdateEvent = {
              action: ShortcutActions["StartPan"],
            };
            PanelEventBus.$emit("shortcuts-update-" + c, e);
          }
        }
      } else {
      }
    },
    spacebarUp(): void {
      if (this.isPanning) {
        for (var c of this.eligibleContexts) {
          if (_.startsWith(c, "graphViewer")) {
            let e: ShortcutUpdateEvent = {
              action: ShortcutActions["EndPan"],
            };
            PanelEventBus.$emit("shortcuts-update-" + c, e);
          }
        }
        this.isPanning = false;
      } else {
        for (var c of this.eligibleContexts) {
          if (_.startsWith(c, "panel")) {
            let e: ShortcutUpdateEvent = {
              action: ShortcutActions["Maximize"],
            };
            PanelEventBus.$emit("shortcuts-update-" + c, e);
          }
        }
      }
      this.spacebarIsPressed = false;
    },
  },
});
</script>
