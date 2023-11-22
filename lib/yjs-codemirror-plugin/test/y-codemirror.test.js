
import * as t from 'lib0/testing'
import * as prng from 'lib0/prng'
import * as math from 'lib0/math'
import * as Y from 'yjs' // eslint-disable-line
import { EditorView, basicSetup } from 'codemirror'
import { EditorState } from '@codemirror/state'

import { applyRandomTests } from 'yjs/tests/testHelper.js'

// @ts-ignore
import { yCollab } from 'y-codemirror.next'

/**
 * @param {any} ydoc
 * @return {EditorView}
 */
const createNewCodemirror = ydoc => {
  const state = EditorState.create({
    doc: ydoc.getText().toString(),
    extensions: [
      basicSetup,
      yCollab(ydoc.getText(), null, { undoManager: false })
    ]
  })
  return new EditorView({ state })
}

let charCounter = 0

/**
 * @type {Array<function(Y.Doc, prng.PRNG, EditorView):{from:number,to:number,insert:string}>}
 */
const trChange = [
  /**
   * @param {Y.Doc} y
   * @param {prng.PRNG} gen
   * @param {EditorView} cm
   */
  (y, gen, cm) => { // insert text
    const from = prng.int32(gen, 0, cm.state.doc.length)
    const insert = charCounter++ + prng.utf16String(gen, 6)
    return {
      from,
      to: from,
      insert
    }
  },
  /**
   * @param {Y.Doc} y
   * @param {prng.PRNG} gen
   * @param {EditorView} cm
   */
  (y, gen, cm) => { // delete text
    const from = prng.int32(gen, 0, cm.state.doc.length)
    const to = from + prng.int32(gen, 0, cm.state.doc.length - from)
    const insert = ''
    return {
      from,
      to,
      insert
    }
  },
  /**
   * @param {Y.Doc} y
   * @param {prng.PRNG} gen
   * @param {EditorView} cm
   */
  (y, gen, cm) => { // replace text
    const from = prng.int32(gen, 0, cm.state.doc.length)
    const to = from + math.min(prng.int32(gen, 0, cm.state.doc.length - from), 3)
    const insert = charCounter++ + prng.word(gen)
    return {
      from,
      to,
      insert
    }
  },
  /**
   * @param {Y.Doc} y
   * @param {prng.PRNG} gen
   * @param {EditorView} cm
   */
  (y, gen, cm) => { // insert paragraph
    const from = prng.int32(gen, 0, cm.state.doc.length)
    const to = from + math.min(prng.int32(gen, 0, cm.state.doc.length - from), 3)
    const insert = '\n'
    return {
      from,
      to,
      insert
    }
  }
]

const cmChanges = [
  /**
   * @param {Y.Doc} y
   * @param {prng.PRNG} gen
   * @param {EditorView} cm
   */
  (y, gen, cm) => { // create a transaction containing 1-4 changes
    const changes = []
    const numOfChanges = prng.int31(gen, 1, 4)
    for (let i = 0; i < numOfChanges; i++) {
      changes.push(prng.oneOf(gen, trChange)(y, gen, cm))
    }
    cm.dispatch({ changes })
  }
]

/**
 * @param {any} result
 */
const checkResult = result => {
  for (let i = 1; i < result.testObjects.length; i++) {
    const p1 = result.testObjects[i - 1].state.doc.toString()
    const p2 = result.testObjects[i].state.doc.toString()
    t.compare(p1, p2)
  }
  charCounter = 0
}

/**
 * @param {t.TestCase} tc
 */
export const testRepeatGenerateProsemirrorChanges2 = tc => {
  checkResult(applyRandomTests(tc, cmChanges, 2, createNewCodemirror))
}

/**
 * @param {t.TestCase} tc
 */
export const testRepeatGenerateProsemirrorChanges3 = tc => {
  checkResult(applyRandomTests(tc, cmChanges, 3, createNewCodemirror))
}

/**
 * @param {t.TestCase} tc
 */
export const testRepeatGenerateProsemirrorChanges30 = tc => {
  checkResult(applyRandomTests(tc, cmChanges, 30, createNewCodemirror))
}

/**
 * @param {t.TestCase} tc
 */
export const testRepeatGenerateProsemirrorChanges40 = tc => {
  checkResult(applyRandomTests(tc, cmChanges, 40, createNewCodemirror))
}

/**
 * @param {t.TestCase} tc
 */
export const testRepeatGenerateProsemirrorChanges70 = tc => {
  checkResult(applyRandomTests(tc, cmChanges, 70, createNewCodemirror))
}

/**
 * @param {t.TestCase} tc
 */
export const testRepeatGenerateProsemirrorChanges100 = tc => {
  checkResult(applyRandomTests(tc, cmChanges, 100, createNewCodemirror))
}

/**
 * @param {t.TestCase} tc
 */
export const testRepeatGenerateProsemirrorChanges300 = tc => {
  checkResult(applyRandomTests(tc, cmChanges, 300, createNewCodemirror))
}
