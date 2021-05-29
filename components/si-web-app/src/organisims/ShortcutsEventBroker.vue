<template>
  <div>
    <slot />
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { PanelEventBus } from "@/atoms/PanelEventBus";
import _ from "lodash";
import { Subject } from "rxjs";
import { fromEvent } from "rxjs";

// @keydown.space="handleSpacebar($event)"

interface KeyboardKey {
  long: string;
  short: string;
}

const spacebarKey: KeyboardKey = {
  long: "Spacebar",
  short: " ",
};

const backspaceKey: KeyboardKey = {
  long: "Backspace",
  short: "Backspace",
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
  DeleteNode = "delete-node",
}

export interface ShortcutUpdateEvent {
  action: ShortcutActions;
  panelId: string;
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

export const SpaceBarEvents = new Subject();
export const BackspaceEvents: Subject<ShortcutUpdateEvent> = new Subject();

const MousecDown = fromEvent(document, "mousedown");
const MousecUp = fromEvent(document, "mouseup");

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
      MousecDown.subscribe(x => this.handleMouseEvents(x as Event));
      MousecUp.subscribe(x => this.handleMouseEvents(x as Event));
    },
    deRegisterEvents(): void {
      PanelEventBus.$off(
        "shortcuts-registration-update",
        this.handleShortcutsRegistrationUpdate,
      );

      document.removeEventListener("keydown", this.handleKeyDown);
      document.removeEventListener("keyup", this.handleKeyUp);
    },
    handleShortcutsRegistrationUpdate(e: ShortcutRegistrationEvent) {
      if (e.context.isActive) {
        if (!this.eligibleContexts.includes(e.context.id))
          this.eligibleContexts.push(e.context.id);
      } else {
        let i = this.eligibleContexts.indexOf(e.context.id);
        this.eligibleContexts = this.eligibleContexts.splice(i, 1);
      }
    },
    handleMouseEvents(e: Event) {
      if (e.type === "mousedown") {
        this.mouseIsClicked = true;
        if (this.spacebarIsPressed && this.mouseIsClicked) {
          this.isPanning = true;
          for (var c of this.eligibleContexts) {
            if (_.startsWith(c, "graphViewer")) {
              const e: ShortcutUpdateEvent = {
                action: ShortcutActions["StartPan"],
                panelId: c,
              };
              SpaceBarEvents.next(e);
            }
          }
        }
      }
      if (e.type === "mouseup") {
        this.mouseIsClicked = false;
        for (var c of this.eligibleContexts) {
          if (_.startsWith(c, "graphViewer")) {
            let e: ShortcutUpdateEvent = {
              action: ShortcutActions["EndPan"],
              panelId: c,
            };
            SpaceBarEvents.next(e);
          }
        }
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

      if (e.key === backspaceKey.long) {
        e.preventDefault();
        this.handleBackspace("keyUp");
      }

      // if (e.key === "f") {
      //   e.preventDefault();
      //   this.handleKeyF();
      // }

      // if (e.key === "a") {
      //   e.preventDefault();
      //   this.handleKeyA();
      // }
    },
    handleSpacebar(e: KeyboardEventKind) {
      if (e == "keyDown") {
        this.spacebarDown();
      } else if (e == "keyUp") {
        this.spacebarUp();
      }
    },
    handleBackspace(e: KeyboardEventKind) {
      if (e == "keyUp") {
        this.backspaceUp();
      }
    },
    spacebarDown(): void {
      this.spacebarIsPressed = true;
    },
    spacebarUp(): void {
      if (this.isPanning) {
        this.isPanning = false;
      }
      // Disable spacebar to maximize
      // } else {
      //   for (var c of this.eligibleContexts) {
      //     if (_.startsWith(c, "panel")) {
      //       let e: ShortcutUpdateEvent = {
      //         action: ShortcutActions["Maximize"],
      //         panelId: c,
      //       };
      //       PanelEventBus.$emit("shortcuts-update-" + c, e);
      //     }
      //   }
      // }
      this.spacebarIsPressed = false;
    },
    handleKeyF(): void {
      // frame selected
    },
    handleKeyA(): void {
      // frame all
    },
    backspaceUp(): void {
      for (var c of this.eligibleContexts) {
        if (_.startsWith(c, "graphViewer")) {
          const e: ShortcutUpdateEvent = {
            action: ShortcutActions["DeleteNode"],
            panelId: c,
          };
          BackspaceEvents.next(e);
        }
      }
    },
  },
});
</script>
