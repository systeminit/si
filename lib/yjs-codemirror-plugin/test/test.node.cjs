// @ts-nocheck

const fs = require('fs')
const path = require('path')
const jsdom = require('jsdom')

const documentContent = fs.readFileSync(path.join(__dirname, '../test.html'))
const { window } = new jsdom.JSDOM(documentContent)

;[
  'window',
  'innerHeight',
  'document',
  'Node',
  'navigator',
  'Text',
  'HTMLElement',
  'MutationObserver'
].forEach(name => {
  global[name] = window[name]
})
document.getSelection = () => ({ })
global.requestAnimationFrame = f => setTimeout(f, 0)
window.Element.prototype.scrollTo = () => {}
global.scrollTo = () => {}

document.createRange = () => ({
  setStart () {},
  setEnd () {},
  getClientRects () {
    return {
      left: 0,
      top: 0,
      right: 0,
      bottom: 0
    }
  },
  getBoundingClientRect () {
    return {
      left: 0,
      top: 0,
      right: 0,
      bottom: 0
    }
  }
})

require('../dist/test.cjs')
