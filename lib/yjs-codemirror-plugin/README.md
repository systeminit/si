# y-codemirror.next

> [CodeMirror 6](https://codemirror.net/) editor binding for [Yjs](https://github.com/yjs/yjs) - [Demo](https://demos.yjs.dev/codemirror.next/codemirror.next.html)

This binding binds a [Y.Text](https://docs.yjs.dev/api/shared-types/y.text) to a CodeMirror editor.

## Features

* Sync CodeMirror 6 editor
* Awareness: Render remote selection ranges and cursors - as a separate plugin
* Shared Undo / Redo (each client has its own undo-/redo-history) - as a separate plugin

![CodeMirror Yjs Demo](https://user-images.githubusercontent.com/5553757/79250004-5ed1ac80-7e7e-11ea-81b8-9f833e2d8e66.gif)

### Example

```js
/* eslint-env browser */

import * as Y from 'yjs'
// @ts-ignore
import { yCollab } from 'y-codemirror.next'
import { WebrtcProvider } from 'y-webrtc'

import { EditorView, basicSetup } from "codemirror";
import { EditorState } from "@codemirror/state";
import { javascript } from '@codemirror/lang-javascript'

import * as random from 'lib0/random'

export const usercolors = [
  { color: '#30bced', light: '#30bced33' },
  { color: '#6eeb83', light: '#6eeb8333' },
  { color: '#ffbc42', light: '#ffbc4233' },
  { color: '#ecd444', light: '#ecd44433' },
  { color: '#ee6352', light: '#ee635233' },
  { color: '#9ac2c9', light: '#9ac2c933' },
  { color: '#8acb88', light: '#8acb8833' },
  { color: '#1be7ff', light: '#1be7ff33' }
]

// select a random color for this user
export const userColor = usercolors[random.uint32() % usercolors.length]

const ydoc = new Y.Doc()
const provider = new WebrtcProvider('codemirror6-demo-room', ydoc)
const ytext = ydoc.getText('codemirror')

const undoManager = new Y.UndoManager(ytext)

provider.awareness.setLocalStateField('user', {
  name: 'Anonymous ' + Math.floor(Math.random() * 100),
  color: userColor.color,
  colorLight: userColor.light
})

const state = EditorState.create({
  doc: ytext.toString(),
  extensions: [
    basicSetup,
    javascript(),
    yCollab(ytext, provider.awareness, { undoManager })
  ]
})

const view = new EditorView({ state, parent: /** @type {HTMLElement} */ (document.querySelector('#editor')) })

```

## API

..

## License

[The MIT License](./LICENSE) © Kevin Jahns
