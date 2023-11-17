export class YUndoManagerConfig {
    /**
     * @param {Y.UndoManager} undoManager
     */
    constructor(undoManager: Y.UndoManager);
    undoManager: Y.UndoManager;
    /**
     * @param {any} origin
     */
    addTrackedOrigin(origin: any): void;
    /**
     * @param {any} origin
     */
    removeTrackedOrigin(origin: any): void;
    /**
     * @return {boolean} Whether a change was undone.
     */
    undo(): boolean;
    /**
     * @return {boolean} Whether a change was redone.
     */
    redo(): boolean;
}
/**
 * @type {cmState.Facet<YUndoManagerConfig, YUndoManagerConfig>}
 */
export const yUndoManagerFacet: cmState.Facet<YUndoManagerConfig, YUndoManagerConfig>;
/**
 * @type {cmState.AnnotationType<YUndoManagerConfig>}
 */
export const yUndoManagerAnnotation: cmState.AnnotationType<YUndoManagerConfig>;
export const yUndoManager: cmView.ViewPlugin<YUndoManagerPluginValue>;
/**
 * @type {cmState.StateCommand}
 */
export const undo: cmState.StateCommand;
/**
 * @type {cmState.StateCommand}
 */
export const redo: cmState.StateCommand;
export function undoDepth(state: cmState.EditorState): number;
export function redoDepth(state: cmState.EditorState): number;
/**
 * Default key bindigs for the undo manager.
 * @type {Array<cmView.KeyBinding>}
 */
export const yUndoManagerKeymap: Array<cmView.KeyBinding>;
import * as Y from "yjs";
import * as cmState from "@codemirror/state";
/**
 * @extends {PluginValue}
 */
declare class YUndoManagerPluginValue {
    /**
     * @param {cmView.EditorView} view
     */
    constructor(view: cmView.EditorView);
    view: cmView.EditorView;
    conf: YUndoManagerConfig;
    _undoManager: Y.UndoManager;
    syncConf: import("./y-sync.js").YSyncConfig;
    /**
     * @type {null | YRange}
     */
    _beforeChangeSelection: null | YRange;
    _mux: import("lib0/mutex.js").mutex;
    _onStackItemAdded: ({ stackItem, changedParentTypes }: {
        stackItem: any;
        changedParentTypes: any;
    }) => void;
    _onStackItemPopped: ({ stackItem }: {
        stackItem: any;
    }) => void;
    /**
     * Do this without mutex, simply use the sync annotation
     */
    _storeSelection: () => void;
    /**
     * @param {cmView.ViewUpdate} update
     */
    update(update: cmView.ViewUpdate): void;
    destroy(): void;
}
import * as cmView from "@codemirror/view";
import { YRange } from "./y-range.js";
export {};
