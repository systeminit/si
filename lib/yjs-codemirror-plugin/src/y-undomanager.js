import * as Y from 'yjs' // eslint-disable-line
import * as cmState from '@codemirror/state'

import * as cmView from '@codemirror/view'
import { ySyncFacet, ySyncAnnotation } from './y-sync.js'
import { YRange } from './y-range.js' // eslint-disable-line
import { createMutex } from 'lib0/mutex'

export class YUndoManagerConfig {
  /**
   * @param {Y.UndoManager} undoManager
   */
  constructor (undoManager) {
    this.undoManager = undoManager
  }

  /**
   * @param {any} origin
   */
  addTrackedOrigin (origin) {
    this.undoManager.addTrackedOrigin(origin)
  }

  /**
   * @param {any} origin
   */
  removeTrackedOrigin (origin) {
    this.undoManager.removeTrackedOrigin(origin)
  }

  /**
   * @return {boolean} Whether a change was undone.
   */
  undo () {
    return this.undoManager.undo() != null
  }

  /**
   * @return {boolean} Whether a change was redone.
   */
  redo () {
    return this.undoManager.redo() != null
  }
}

/**
 * @type {cmState.Facet<YUndoManagerConfig, YUndoManagerConfig>}
 */
export const yUndoManagerFacet = cmState.Facet.define({
  combine (inputs) {
    return inputs[inputs.length - 1]
  }
})

/**
 * @type {cmState.AnnotationType<YUndoManagerConfig>}
 */
export const yUndoManagerAnnotation = cmState.Annotation.define()

/**
 * @extends {PluginValue}
 */
class YUndoManagerPluginValue {
  /**
   * @param {cmView.EditorView} view
   */
  constructor (view) {
    this.view = view
    this.conf = view.state.facet(yUndoManagerFacet)
    this._undoManager = this.conf.undoManager
    this.syncConf = view.state.facet(ySyncFacet)
    /**
     * @type {null | YRange}
     */
    this._beforeChangeSelection = null
    this._mux = createMutex()

    this._onStackItemAdded = ({ stackItem, changedParentTypes }) => {
      // only store metadata if this type was affected
      if (changedParentTypes.has(this.syncConf.ytext) && this._beforeChangeSelection && !stackItem.meta.has(this)) { // do not overwrite previous stored selection
        stackItem.meta.set(this, this._beforeChangeSelection)
      }
    }
    this._onStackItemPopped = ({ stackItem }) => {
      const sel = stackItem.meta.get(this)
      if (sel) {
        const selection = this.syncConf.fromYRange(sel)
        view.dispatch(view.state.update({ selection }))
        this._storeSelection()
      }
    }
    /**
     * Do this without mutex, simply use the sync annotation
     */
    this._storeSelection = () => {
      // store the selection before the change is applied so we can restore it with the undo manager.
      this._beforeChangeSelection = this.syncConf.toYRange(this.view.state.selection.main)
    }
    this._undoManager.on('stack-item-added', this._onStackItemAdded)
    this._undoManager.on('stack-item-popped', this._onStackItemPopped)
    this._undoManager.addTrackedOrigin(this.syncConf)
  }

  /**
   * @param {cmView.ViewUpdate} update
   */
  update (update) {
    if (update.selectionSet && (update.transactions.length === 0 || update.transactions[0].annotation(ySyncAnnotation) !== this.syncConf)) {
      // This only works when YUndoManagerPlugin is included before the sync plugin
      this._storeSelection()
    }
  }

  destroy () {
    this._undoManager.off('stack-item-added', this._onStackItemAdded)
    this._undoManager.off('stack-item-popped', this._onStackItemPopped)
    this._undoManager.removeTrackedOrigin(this.syncConf)
  }
}
export const yUndoManager = cmView.ViewPlugin.fromClass(YUndoManagerPluginValue)

/**
 * @type {cmState.StateCommand}
 */
export const undo = ({ state, dispatch }) =>
  state.facet(yUndoManagerFacet).undo() || true

/**
 * @type {cmState.StateCommand}
 */
export const redo = ({ state, dispatch }) =>
  state.facet(yUndoManagerFacet).redo() || true

/**
 * @param {cmState.EditorState} state
 * @return {number}
 */
export const undoDepth = state => state.facet(yUndoManagerFacet).undoManager.undoStack.length

/**
 * @param {cmState.EditorState} state
 * @return {number}
 */
export const redoDepth = state => state.facet(yUndoManagerFacet).undoManager.redoStack.length

/**
 * Default key bindigs for the undo manager.
 * @type {Array<cmView.KeyBinding>}
 */
export const yUndoManagerKeymap = [
  { key: 'Mod-z', run: undo, preventDefault: true },
  { key: 'Mod-y', mac: 'Mod-Shift-z', run: redo, preventDefault: true },
  { key: 'Mod-Shift-z', run: redo, preventDefault: true }
]
