export const yRemoteSelectionsTheme: cmState.Extension;
export class YRemoteSelectionsPluginValue {
    /**
     * @param {cmView.EditorView} view
     */
    constructor(view: cmView.EditorView);
    conf: import("./y-sync.js").YSyncConfig;
    _listener: ({ added, updated, removed }: {
        added: any;
        updated: any;
        removed: any;
    }, s: any, t: any) => void;
    _awareness: any;
    /**
     * @type {cmView.DecorationSet}
     */
    decorations: cmView.DecorationSet;
    destroy(): void;
    /**
     * @param {cmView.ViewUpdate} update
     */
    update(update: cmView.ViewUpdate): void;
}
export const yRemoteSelections: cmView.ViewPlugin<YRemoteSelectionsPluginValue>;
import * as cmState from "@codemirror/state";
import * as cmView from "@codemirror/view";
