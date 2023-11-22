export class YSyncConfig {
    constructor(ytext: any, awareness: any, getUserInfo: any);
    ytext: any;
    awareness: any;
    undoManager: Y.UndoManager;
    getUserInfo: any;
    /**
     * Helper function to transform an absolute index position to a Yjs-based relative position
     * (https://docs.yjs.dev/api/relative-positions).
     *
     * A relative position can be transformed back to an absolute position even after the document has changed. The position is
     * automatically adapted. This does not require any position transformations. Relative positions are computed based on
     * the internal Yjs document model. Peers that share content through Yjs are guaranteed that their positions will always
     * synced up when using relatve positions.
     *
     * ```js
     * import { ySyncFacet } from 'y-codemirror'
     *
     * ..
     * const ysync = view.state.facet(ySyncFacet)
     * // transform an absolute index position to a ypos
     * const ypos = ysync.getYPos(3)
     * // transform the ypos back to an absolute position
     * ysync.fromYPos(ypos) // => 3
     * ```
     *
     * It cannot be guaranteed that absolute index positions can be synced up between peers.
     * This might lead to undesired behavior when implementing features that require that all peers see the
     * same marked range (e.g. a comment plugin).
     *
     * @param {number} pos
     * @param {number} [assoc]
     */
    toYPos(pos: number, assoc?: number): Y.RelativePosition;
    /**
     * @param {Y.RelativePosition | Object} rpos
     */
    fromYPos(rpos: Y.RelativePosition | any): {
        pos: number;
        assoc: number;
    };
    /**
     * @param {cmState.SelectionRange} range
     * @return {YRange}
     */
    toYRange(range: cmState.SelectionRange): YRange;
    /**
     * @param {YRange} yrange
     */
    fromYRange(yrange: YRange): cmState.SelectionRange;
}
/**
 * @type {cmState.Facet<YSyncConfig, YSyncConfig>}
 */
export const ySyncFacet: cmState.Facet<YSyncConfig, YSyncConfig>;
/**
 * @type {cmState.AnnotationType<YSyncConfig>}
 */
export const ySyncAnnotation: cmState.AnnotationType<YSyncConfig>;
export const ySync: cmView.ViewPlugin<YSyncPluginValue>;
import * as Y from "yjs";
import * as cmState from "@codemirror/state";
import { YRange } from "./y-range.js";
/**
 * @extends {PluginValue}
 */
declare class YSyncPluginValue {
    /**
     * @param {cmView.EditorView} view
     */
    constructor(view: cmView.EditorView);
    view: cmView.EditorView;
    conf: YSyncConfig;
    _observer: (event: any, tr: any) => void;
    _ytext: any;
    /**
     * @param {cmView.ViewUpdate} update
     */
    update(update: cmView.ViewUpdate): void;
    destroy(): void;
}
import * as cmView from "@codemirror/view";
export {};
