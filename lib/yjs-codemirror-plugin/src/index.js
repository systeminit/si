
import * as Y from 'yjs' // eslint-disable-line
import * as cmView from '@codemirror/view'
import * as cmState from '@codemirror/state' // eslint-disable-line

import { YRange } from './y-range.js'
import { ySync, ySyncFacet, YSyncConfig } from './y-sync.js'
import { yRemoteSelections, yRemoteSelectionsTheme } from './y-remote-selections.js'
import { yUndoManager, yUndoManagerFacet, YUndoManagerConfig, undo, redo, yUndoManagerKeymap } from './y-undomanager.js'

export { YRange, yRemoteSelections, yRemoteSelectionsTheme, ySync, ySyncFacet, YSyncConfig, yUndoManagerKeymap }

/**
 * @param {Y.Text} ytext
 * @param {any} awareness
 * @param {Object} [opts]
 * @param {Y.UndoManager | false} [opts.undoManager] Set undoManager to false to disable the undo-redo plugin
 * @param {((u: any) => { name?: String, colorLight?: String, color?: String })} [opts.getUserInfo] Optional fn to take user awareness state info and transform into name + color(s)
 * @return {cmState.Extension}
 */
export const yCollab = (ytext, awareness, opts) => {
  // types not quite right... the jsdoc type stuff is awkward
  const undoManager = opts?.undoManager ?? new Y.UndoManager(ytext)
  const getUserInfo = opts?.getUserInfo
  const ySyncConfig = new YSyncConfig(ytext, awareness, getUserInfo)
  const plugins = [
    ySyncFacet.of(ySyncConfig),
    ySync
  ]

  if (awareness) {
    plugins.push(
      yRemoteSelectionsTheme,
      yRemoteSelections
    )
  }
  if (undoManager !== false) {
    // By default, only track changes that are produced by the sync plugin (local edits)
    plugins.push(
      yUndoManagerFacet.of(new YUndoManagerConfig(undoManager)),
      yUndoManager,
      cmView.EditorView.domEventHandlers({
        beforeinput (e, view) {
          if (e.inputType === 'historyUndo') return undo(view)
          if (e.inputType === 'historyRedo') return redo(view)
          return false
        }
      })
    )
  }
  return plugins
}
