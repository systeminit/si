'use strict';

Object.defineProperty(exports, '__esModule', { value: true });

var Y = require('yjs');
var cmView = require('@codemirror/view');
var cmState = require('@codemirror/state');
var dom = require('lib0/dist/dom.cjs');
var pair = require('lib0/dist/pair.cjs');
var math = require('lib0/dist/math.cjs');
var mutex = require('lib0/dist/mutex.cjs');

function _interopNamespace(e) {
  if (e && e.__esModule) return e;
  var n = Object.create(null);
  if (e) {
    Object.keys(e).forEach(function (k) {
      if (k !== 'default') {
        var d = Object.getOwnPropertyDescriptor(e, k);
        Object.defineProperty(n, k, d.get ? d : {
          enumerable: true,
          get: function () { return e[k]; }
        });
      }
    });
  }
  n["default"] = e;
  return Object.freeze(n);
}

var Y__namespace = /*#__PURE__*/_interopNamespace(Y);
var cmView__namespace = /*#__PURE__*/_interopNamespace(cmView);
var cmState__namespace = /*#__PURE__*/_interopNamespace(cmState);
var dom__namespace = /*#__PURE__*/_interopNamespace(dom);
var pair__namespace = /*#__PURE__*/_interopNamespace(pair);
var math__namespace = /*#__PURE__*/_interopNamespace(math);

/**
 * Defines a range on text using relative positions that can be transformed back to
 * absolute positions. (https://docs.yjs.dev/api/relative-positions)
 */
class YRange {
  /**
   * @param {Y.RelativePosition} yanchor
   * @param {Y.RelativePosition} yhead
   */
  constructor (yanchor, yhead) {
    this.yanchor = yanchor;
    this.yhead = yhead;
  }

  /**
   * @returns {any}
   */
  toJSON () {
    return {
      yanchor: Y__namespace.relativePositionToJSON(this.yanchor),
      yhead: Y__namespace.relativePositionToJSON(this.yhead)
    }
  }

  /**
   * @param {any} json
   * @return {YRange}
   */
  static fromJSON (json) {
    return new YRange(Y__namespace.createRelativePositionFromJSON(json.yanchor), Y__namespace.createRelativePositionFromJSON(json.yhead))
  }
}

class YSyncConfig {
  constructor (ytext, awareness, getUserInfo) {
    this.ytext = ytext;
    this.awareness = awareness;
    this.undoManager = new Y__namespace.UndoManager(ytext);
    this.getUserInfo = getUserInfo;
  }

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
  toYPos (pos, assoc = 0) {
    return Y__namespace.createRelativePositionFromTypeIndex(this.ytext, pos, assoc)
  }

  /**
   * @param {Y.RelativePosition | Object} rpos
   */
  fromYPos (rpos) {
    const pos = Y__namespace.createAbsolutePositionFromRelativePosition(Y__namespace.createRelativePositionFromJSON(rpos), this.ytext.doc);
    if (pos == null || pos.type !== this.ytext) {
      throw new Error('[y-codemirror] The position you want to retrieve was created by a different document')
    }
    return {
      pos: pos.index,
      assoc: pos.assoc
    }
  }

  /**
   * @param {cmState.SelectionRange} range
   * @return {YRange}
   */
  toYRange (range) {
    const assoc = range.assoc;
    const yanchor = this.toYPos(range.anchor, assoc);
    const yhead = this.toYPos(range.head, assoc);
    return new YRange(yanchor, yhead)
  }

  /**
   * @param {YRange} yrange
   */
  fromYRange (yrange) {
    const anchor = this.fromYPos(yrange.yanchor);
    const head = this.fromYPos(yrange.yhead);
    if (anchor.pos === head.pos) {
      return cmState__namespace.EditorSelection.cursor(head.pos, head.assoc)
    }
    return cmState__namespace.EditorSelection.range(anchor.pos, head.pos)
  }
}

/**
 * @type {cmState.Facet<YSyncConfig, YSyncConfig>}
 */
const ySyncFacet = cmState__namespace.Facet.define({
  combine (inputs) {
    return inputs[inputs.length - 1]
  }
});

/**
 * @type {cmState.AnnotationType<YSyncConfig>}
 */
const ySyncAnnotation = cmState__namespace.Annotation.define();

/**
 * @extends {PluginValue}
 */
class YSyncPluginValue {
  /**
   * @param {cmView.EditorView} view
   */
  constructor (view) {
    this.view = view;
    this.conf = view.state.facet(ySyncFacet);
    this._observer = (event, tr) => {
      if (tr.origin !== this.conf) {
        const delta = event.delta;
        const changes = [];
        let pos = 0;
        for (let i = 0; i < delta.length; i++) {
          const d = delta[i];
          if (d.insert != null) {
            changes.push({ from: pos, to: pos, insert: d.insert });
          } else if (d.delete != null) {
            changes.push({ from: pos, to: pos + d.delete, insert: '' });
            pos += d.delete;
          } else {
            pos += d.retain;
          }
        }
        view.dispatch({ changes, annotations: [ySyncAnnotation.of(this.conf)] });
      }
    };
    this._ytext = this.conf.ytext;
    this._ytext.observe(this._observer);
  }

  /**
   * @param {cmView.ViewUpdate} update
   */
  update (update) {
    if (!update.docChanged || (update.transactions.length > 0 && update.transactions[0].annotation(ySyncAnnotation) === this.conf)) {
      return
    }
    const ytext = this.conf.ytext;
    ytext.doc.transact(() => {
      /**
       * This variable adjusts the fromA position to the current position in the Y.Text type.
       */
      let adj = 0;
      update.changes.iterChanges((fromA, toA, fromB, toB, insert) => {
        const insertText = insert.sliceString(0, insert.length, '\n');
        if (fromA !== toA) {
          ytext.delete(fromA + adj, toA - fromA);
        }
        if (insertText.length > 0) {
          ytext.insert(fromA + adj, insertText);
        }
        adj += insertText.length - (toA - fromA);
      });
    }, this.conf);
  }

  destroy () {
    this._ytext.unobserve(this._observer);
  }
}

const ySync = cmView__namespace.ViewPlugin.fromClass(YSyncPluginValue);

const yRemoteSelectionsTheme = cmView__namespace.EditorView.baseTheme({
  '.cm-ySelection': {
  },
  '.cm-yLineSelection': {
    padding: 0,
    margin: '0px 2px 0px 4px'
  },
  '.cm-ySelectionCaret': {
    position: 'relative',
    borderLeft: '1px solid black',
    borderRight: '1px solid black',
    marginLeft: '-1px',
    marginRight: '-1px',
    boxSizing: 'border-box',
    display: 'inline'
  },
  '.cm-ySelectionCaretDot': {
    borderRadius: '50%',
    position: 'absolute',
    width: '.4em',
    height: '.4em',
    top: '-.2em',
    left: '-.2em',
    backgroundColor: 'inherit',
    transition: 'transform .3s ease-in-out',
    boxSizing: 'border-box'
  },
  '.cm-ySelectionCaret:hover > .cm-ySelectionCaretDot': {
    transformOrigin: 'bottom center',
    transform: 'scale(0)'
  },
  '.cm-ySelectionInfo': {
    position: 'absolute',
    top: '-1.05em',
    left: '-1px',
    fontSize: '.75em',
    fontFamily: 'serif',
    fontStyle: 'normal',
    fontWeight: 'normal',
    lineHeight: 'normal',
    userSelect: 'none',
    color: 'white',
    paddingLeft: '2px',
    paddingRight: '2px',
    zIndex: 101,
    transition: 'opacity .3s ease-in-out',
    backgroundColor: 'inherit',
    // these should be separate
    opacity: 0,
    transitionDelay: '0s',
    whiteSpace: 'nowrap'
  },
  '.cm-ySelectionCaret:hover > .cm-ySelectionInfo': {
    opacity: 1,
    transitionDelay: '0s'
  }
});

/**
 * @todo specify the users that actually changed. Currently, we recalculate positions for every user.
 * @type {cmState.AnnotationType<Array<number>>}
 */
const yRemoteSelectionsAnnotation = cmState__namespace.Annotation.define();

class YRemoteCaretWidget extends cmView__namespace.WidgetType {
  /**
   * @param {string} color
   * @param {string} name
   */
  constructor (color, name) {
    super();
    this.color = color;
    this.name = name;
  }

  toDOM () {
    return /** @type {HTMLElement} */ (dom__namespace.element('span', [pair__namespace.create('class', 'cm-ySelectionCaret'), pair__namespace.create('style', `background-color: ${this.color}; border-color: ${this.color}`)], [
      dom__namespace.text('\u2060'),
      dom__namespace.element('div', [
        pair__namespace.create('class', 'cm-ySelectionCaretDot')
      ]),
      dom__namespace.text('\u2060'),
      dom__namespace.element('div', [
        pair__namespace.create('class', 'cm-ySelectionInfo')
      ], [
        dom__namespace.text(this.name)
      ]),
      dom__namespace.text('\u2060')
    ]))
  }

  eq (widget) {
    return widget.color === this.color
  }

  compare (widget) {
    return widget.color === this.color
  }

  updateDOM () {
    return false
  }

  get estimatedHeight () { return -1 }

  ignoreEvent () {
    return true
  }
}

class YRemoteSelectionsPluginValue {
  /**
   * @param {cmView.EditorView} view
   */
  constructor (view) {
    this.conf = view.state.facet(ySyncFacet);
    this._listener = ({ added, updated, removed }, s, t) => {
      const clients = added.concat(updated).concat(removed);
      if (clients.findIndex(id => id !== this.conf.awareness.doc.clientID) >= 0) {
        view.dispatch({ annotations: [yRemoteSelectionsAnnotation.of([])] });
      }
    };
    this._awareness = this.conf.awareness;
    this._awareness.on('change', this._listener);
    /**
     * @type {cmView.DecorationSet}
     */
    this.decorations = cmState__namespace.RangeSet.of([]);
  }

  destroy () {
    this._awareness.off('change', this._listener);
  }

  /**
   * @param {cmView.ViewUpdate} update
   */
  update (update) {
    const ytext = this.conf.ytext;
    const ydoc = /** @type {Y.Doc} */ (ytext.doc);
    const awareness = this.conf.awareness;
    /**
     * @type {Array<cmState.Range<cmView.Decoration>>}
     */
    const decorations = [];
    const localAwarenessState = this.conf.awareness.getLocalState();

    // set local awareness state (update cursors)
    if (localAwarenessState != null) {
      const hasFocus = update.view.hasFocus && update.view.dom.ownerDocument.hasFocus();
      const sel = hasFocus ? update.state.selection.main : null;
      const currentAnchor = localAwarenessState.cursor == null ? null : Y__namespace.createRelativePositionFromJSON(localAwarenessState.cursor.anchor);
      const currentHead = localAwarenessState.cursor == null ? null : Y__namespace.createRelativePositionFromJSON(localAwarenessState.cursor.head);

      if (sel != null) {
        const anchor = Y__namespace.createRelativePositionFromTypeIndex(ytext, sel.anchor);
        const head = Y__namespace.createRelativePositionFromTypeIndex(ytext, sel.head);
        if (localAwarenessState.cursor == null || !Y__namespace.compareRelativePositions(currentAnchor, anchor) || !Y__namespace.compareRelativePositions(currentHead, head)) {
          awareness.setLocalStateField('cursor', {
            anchor,
            head
          });
        }
      } else if (localAwarenessState.cursor != null && hasFocus) {
        awareness.setLocalStateField('cursor', null);
      }
    }

    // update decorations (remote selections)
    awareness.getStates().forEach((state, clientid) => {
      if (clientid === awareness.doc.clientID) {
        return
      }
      const cursor = state.cursor;
      if (cursor == null || cursor.anchor == null || cursor.head == null) {
        return
      }
      const anchor = Y__namespace.createAbsolutePositionFromRelativePosition(cursor.anchor, ydoc);
      const head = Y__namespace.createAbsolutePositionFromRelativePosition(cursor.head, ydoc);
      if (anchor == null || head == null || anchor.type !== ytext || head.type !== ytext) {
        return
      }

      const userColorAndName = this.conf.getUserInfo ? this.conf.getUserInfo(state.user) : state.user;
      const color = userColorAndName?.color || '#30bced';
      const name = userColorAndName?.name || 'Anonymous';
      const colorLight = userColorAndName.colorLight || color + '33';
      const start = math__namespace.min(anchor.index, head.index);
      const end = math__namespace.max(anchor.index, head.index);
      const startLine = update.view.state.doc.lineAt(start);
      const endLine = update.view.state.doc.lineAt(end);
      if (startLine.number === endLine.number) {
        // selected content in a single line.
        decorations.push({
          from: start,
          to: end,
          value: cmView__namespace.Decoration.mark({
            attributes: { style: `background-color: ${colorLight}` },
            class: 'cm-ySelection'
          })
        });
      } else {
        // selected content in multiple lines
        // first, render text-selection in the first line
        decorations.push({
          from: start,
          to: startLine.from + startLine.length,
          value: cmView__namespace.Decoration.mark({
            attributes: { style: `background-color: ${colorLight}` },
            class: 'cm-ySelection'
          })
        });
        // render text-selection in the last line
        decorations.push({
          from: endLine.from,
          to: end,
          value: cmView__namespace.Decoration.mark({
            attributes: { style: `background-color: ${colorLight}` },
            class: 'cm-ySelection'
          })
        });
        for (let i = startLine.number + 1; i < endLine.number; i++) {
          const linePos = update.view.state.doc.line(i).from;
          decorations.push({
            from: linePos,
            to: linePos,
            value: cmView__namespace.Decoration.line({
              attributes: { style: `background-color: ${colorLight}`, class: 'cm-yLineSelection' }
            })
          });
        }
      }
      decorations.push({
        from: head.index,
        to: head.index,
        value: cmView__namespace.Decoration.widget({
          side: head.index - anchor.index > 0 ? -1 : 1, // the local cursor should be rendered outside the remote selection
          block: false,
          widget: new YRemoteCaretWidget(color, name)
        })
      });
    });
    this.decorations = cmView__namespace.Decoration.set(decorations, true);
  }
}

const yRemoteSelections = cmView__namespace.ViewPlugin.fromClass(YRemoteSelectionsPluginValue, {
  decorations: v => v.decorations
});

class YUndoManagerConfig {
  /**
   * @param {Y.UndoManager} undoManager
   */
  constructor (undoManager) {
    this.undoManager = undoManager;
  }

  /**
   * @param {any} origin
   */
  addTrackedOrigin (origin) {
    this.undoManager.addTrackedOrigin(origin);
  }

  /**
   * @param {any} origin
   */
  removeTrackedOrigin (origin) {
    this.undoManager.removeTrackedOrigin(origin);
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
const yUndoManagerFacet = cmState__namespace.Facet.define({
  combine (inputs) {
    return inputs[inputs.length - 1]
  }
});

/**
 * @type {cmState.AnnotationType<YUndoManagerConfig>}
 */
cmState__namespace.Annotation.define();

/**
 * @extends {PluginValue}
 */
class YUndoManagerPluginValue {
  /**
   * @param {cmView.EditorView} view
   */
  constructor (view) {
    this.view = view;
    this.conf = view.state.facet(yUndoManagerFacet);
    this._undoManager = this.conf.undoManager;
    this.syncConf = view.state.facet(ySyncFacet);
    /**
     * @type {null | YRange}
     */
    this._beforeChangeSelection = null;
    this._mux = mutex.createMutex();

    this._onStackItemAdded = ({ stackItem, changedParentTypes }) => {
      // only store metadata if this type was affected
      if (changedParentTypes.has(this.syncConf.ytext) && this._beforeChangeSelection && !stackItem.meta.has(this)) { // do not overwrite previous stored selection
        stackItem.meta.set(this, this._beforeChangeSelection);
      }
    };
    this._onStackItemPopped = ({ stackItem }) => {
      const sel = stackItem.meta.get(this);
      if (sel) {
        const selection = this.syncConf.fromYRange(sel);
        view.dispatch(view.state.update({ selection }));
        this._storeSelection();
      }
    };
    /**
     * Do this without mutex, simply use the sync annotation
     */
    this._storeSelection = () => {
      // store the selection before the change is applied so we can restore it with the undo manager.
      this._beforeChangeSelection = this.syncConf.toYRange(this.view.state.selection.main);
    };
    this._undoManager.on('stack-item-added', this._onStackItemAdded);
    this._undoManager.on('stack-item-popped', this._onStackItemPopped);
    this._undoManager.addTrackedOrigin(this.syncConf);
  }

  /**
   * @param {cmView.ViewUpdate} update
   */
  update (update) {
    if (update.selectionSet && (update.transactions.length === 0 || update.transactions[0].annotation(ySyncAnnotation) !== this.syncConf)) {
      // This only works when YUndoManagerPlugin is included before the sync plugin
      this._storeSelection();
    }
  }

  destroy () {
    this._undoManager.off('stack-item-added', this._onStackItemAdded);
    this._undoManager.off('stack-item-popped', this._onStackItemPopped);
    this._undoManager.removeTrackedOrigin(this.syncConf);
  }
}
const yUndoManager = cmView__namespace.ViewPlugin.fromClass(YUndoManagerPluginValue);

/**
 * @type {cmState.StateCommand}
 */
const undo = ({ state, dispatch }) =>
  state.facet(yUndoManagerFacet).undo() || true;

/**
 * @type {cmState.StateCommand}
 */
const redo = ({ state, dispatch }) =>
  state.facet(yUndoManagerFacet).redo() || true;

/**
 * Default key bindigs for the undo manager.
 * @type {Array<cmView.KeyBinding>}
 */
const yUndoManagerKeymap = [
  { key: 'Mod-z', run: undo, preventDefault: true },
  { key: 'Mod-y', mac: 'Mod-Shift-z', run: redo, preventDefault: true },
  { key: 'Mod-Shift-z', run: redo, preventDefault: true }
];

/**
 * @param {Y.Text} ytext
 * @param {any} awareness
 * @param {Object} [opts]
 * @param {Y.UndoManager | false} [opts.undoManager] Set undoManager to false to disable the undo-redo plugin
 * @param {((u: any) => { name?: String, colorLight?: String, color?: String })} [opts.getUserInfo] Optional fn to take user awareness state info and transform into name + color(s)
 * @return {cmState.Extension}
 */
const yCollab = (ytext, awareness, opts) => {
  // types not quite right... the jsdoc type stuff is awkward
  const undoManager = opts?.undoManager ?? new Y__namespace.UndoManager(ytext);
  const getUserInfo = opts?.getUserInfo;
  const ySyncConfig = new YSyncConfig(ytext, awareness, getUserInfo);
  const plugins = [
    ySyncFacet.of(ySyncConfig),
    ySync
  ];

  if (awareness) {
    plugins.push(
      yRemoteSelectionsTheme,
      yRemoteSelections
    );
  }
  if (undoManager !== false) {
    // By default, only track changes that are produced by the sync plugin (local edits)
    plugins.push(
      yUndoManagerFacet.of(new YUndoManagerConfig(undoManager)),
      yUndoManager,
      cmView__namespace.EditorView.domEventHandlers({
        beforeinput (e, view) {
          if (e.inputType === 'historyUndo') return undo(view)
          if (e.inputType === 'historyRedo') return redo(view)
          return false
        }
      })
    );
  }
  return plugins
};

exports.YRange = YRange;
exports.YSyncConfig = YSyncConfig;
exports.yCollab = yCollab;
exports.yRemoteSelections = yRemoteSelections;
exports.yRemoteSelectionsTheme = yRemoteSelectionsTheme;
exports.ySync = ySync;
exports.ySyncFacet = ySyncFacet;
exports.yUndoManagerKeymap = yUndoManagerKeymap;
//# sourceMappingURL=y-codemirror.cjs.map
