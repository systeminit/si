// import Vue from "vue";
//
// export const PanelEventBus = new Vue();
//
// document.addEventListener("keydown", function () {
//   PanelEventBus.$emit("shortcut", event);
// });
//
// export function emitEditorErrorMessage(payload: string) {
//   PanelEventBus.$emit("editor-error-message", payload);
// }
//
// export interface IPropChangeEvent {
//   registryProperty: { path: string[] };
//   value:
//     | string
//     | number
//     | Record<string, any>
//     | Record<string, any>[]
//     | undefined;
//   editKind: "edit" | "add" | "delete";
// }
//
// export function emitPropChangeEvent(
//   registryProperty: { path: string[] },
//   entityId: string,
//   editKind: IPropChangeEvent["editKind"],
//   value: IPropChangeEvent["value"],
// ) {
//   let eventName = `${entityId}-${registryProperty.path.join("-")}`;
//   let event: IPropChangeEvent = {
//     registryProperty,
//     value,
//     editKind,
//   };
//   PanelEventBus.$emit(eventName, event);
// }
//
// export function onPropChangeEvent(
//   registryProperty: { path: string[] },
//   entityId: string,
//   callback: (event: IPropChangeEvent) => void,
// ) {
//   let eventName = `${entityId}-${registryProperty.path.join("-")}`;
//   PanelEventBus.$on(eventName, callback);
// }
//
// export function offPropChangeEvent(
//   registryProperty: { path: string[] },
//   entityId: string,
//   callback: (event: IPropChangeEvent) => void,
// ) {
//   let eventName = `${entityId}-${registryProperty.path.join("-")}`;
//   PanelEventBus.$off(eventName, callback);
// }
//
// export function onPropMapAddEvent(
//   registryProperty: { path: string[] },
//   entityId: string,
//   callback: (event: IPropChangeEvent) => void,
// ) {
//   let eventName = `${entityId}-${registryProperty.path.join("-")}-add`;
//   PanelEventBus.$on(eventName, callback);
// }
//
// export function offPropMapAddEvent(
//   registryProperty: { path: string[] },
//   entityId: string,
//   callback: (event: IPropChangeEvent) => void,
// ) {
//   let eventName = `${entityId}-${registryProperty.path.join("-")}-add`;
//   PanelEventBus.$off(eventName, callback);
// }
//
// export function emitPropMapAddEvent(
//   registryProperty: { path: string[] },
//   entityId: string,
//   editKind: IPropChangeEvent["editKind"],
//   value: IPropChangeEvent["value"],
// ) {
//   let eventName = `${entityId}-${registryProperty.path
//     .slice(0, registryProperty.path.length - 1)
//     .join("-")}-add`;
//   let event: IPropChangeEvent = {
//     registryProperty,
//     value,
//     editKind,
//   };
//   PanelEventBus.$emit(eventName, event);
// }
//
// export function onPropMapRemoveEvent(
//   registryProperty: { path: string[] },
//   entityId: string,
//   callback: (event: IPropChangeEvent) => void,
// ) {
//   let eventName = `${entityId}-${registryProperty.path.join("-")}-remove`;
//   PanelEventBus.$on(eventName, callback);
// }
//
// export function offPropMapRemoveEvent(
//   registryProperty: { path: string[] },
//   entityId: string,
//   callback: (event: IPropChangeEvent) => void,
// ) {
//   let eventName = `${entityId}-${registryProperty.path.join("-")}-remove`;
//   PanelEventBus.$off(eventName, callback);
// }
//
// export function emitPropMapRemoveEvent(
//   registryProperty: { path: string[] },
//   entityId: string,
//   editKind: IPropChangeEvent["editKind"],
//   value: IPropChangeEvent["value"],
// ) {
//   let eventName = `${entityId}-${registryProperty.path
//     .slice(0, registryProperty.path.length - 1)
//     .join("-")}-remove`;
//   let event: IPropChangeEvent = {
//     registryProperty,
//     value,
//     editKind,
//   };
//   PanelEventBus.$emit(eventName, event);
// }
//
// export function onPropRepeatedAddEvent(
//   registryProperty: { path: string[] },
//   entityId: string,
//   callback: (event: IPropChangeEvent) => void,
// ) {
//   let eventName = `${entityId}-${registryProperty.path.join("-")}-add`;
//   PanelEventBus.$on(eventName, callback);
// }
//
// export function offPropRepeatedAddEvent(
//   registryProperty: { path: string[] },
//   entityId: string,
//   callback: (event: IPropChangeEvent) => void,
// ) {
//   let eventName = `${entityId}-${registryProperty.path.join("-")}-add`;
//   PanelEventBus.$off(eventName, callback);
// }
//
// export function emitPropRepeatedAddEvent(
//   registryProperty: { path: string[] },
//   entityId: string,
//   editKind: IPropChangeEvent["editKind"],
//   value: IPropChangeEvent["value"],
// ) {
//   let eventName = `${entityId}-${registryProperty.path.join("-")}-add`;
//   let event: IPropChangeEvent = {
//     registryProperty,
//     value,
//     editKind,
//   };
//   console.log("emitting prop repeated add event", { eventName, event });
//   PanelEventBus.$emit(eventName, event);
// }
//
// export function onPropRepeatedRemoveEvent(
//   registryProperty: { path: string[] },
//   entityId: string,
//   callback: (event: IPropChangeEvent) => void,
// ) {
//   let eventName = `${entityId}-${registryProperty.path.join("-")}-remove`;
//   PanelEventBus.$on(eventName, callback);
// }
//
// export function offPropRepeatedRemoveEvent(
//   registryProperty: { path: string[] },
//   entityId: string,
//   callback: (event: IPropChangeEvent) => void,
// ) {
//   let eventName = `${entityId}-${registryProperty.path.join("-")}-remove`;
//   PanelEventBus.$off(eventName, callback);
// }
//
// export function emitPropRepeatedRemoveEvent(
//   registryProperty: { path: string[] },
//   entityId: string,
//   editKind: IPropChangeEvent["editKind"],
//   value: IPropChangeEvent["value"],
// ) {
//   let eventName = `${entityId}-${registryProperty.path.join("-")}-remove`;
//   let event: IPropChangeEvent = {
//     registryProperty,
//     value,
//     editKind,
//   };
//   PanelEventBus.$emit(eventName, event);
// }
// // document.addEventListener("keydown", function() {
// //   PanelEventBus.$emit("shortcut", event);
// // });
//
// function publishPanelViewportUpdateEvent() {
//   // ALEX: DISABLED (please keep arround)
//   // PanelEventBus.$emit("panel-viewport-update", true);
// }
//
// function registerPanelViewportUpdateEvents() {
//   PanelEventBus.$on("panel-layout-update", publishPanelViewportUpdateEvent);
//   PanelEventBus.$on("maximize-container", publishPanelViewportUpdateEvent);
//   PanelEventBus.$on("maximize-full", publishPanelViewportUpdateEvent);
//   PanelEventBus.$on("minimize-container", publishPanelViewportUpdateEvent);
//   PanelEventBus.$on("panel-created", publishPanelViewportUpdateEvent);
//   PanelEventBus.$on("panel-deleted", publishPanelViewportUpdateEvent);
//   window.addEventListener("resize", publishPanelViewportUpdateEvent);
// }
//
// registerPanelViewportUpdateEvents();
