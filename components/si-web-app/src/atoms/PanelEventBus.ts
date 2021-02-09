import Vue from "vue";

export const PanelEventBus = new Vue();

document.addEventListener("keydown", function() {
  PanelEventBus.$emit("shortcut", event);
});
