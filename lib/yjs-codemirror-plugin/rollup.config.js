import nodeResolve from '@rollup/plugin-node-resolve'
import commonjs from '@rollup/plugin-commonjs'
import { terser } from 'rollup-plugin-terser'

/*
const customModules = new Set([
  'y-websocket',
  'y-codemirror',
  'y-ace',
  'y-textarea',
  'y-quill',
  'y-dom',
  'y-prosemirror'
])
/**
 * @type {Set<any>}
 *
const customLibModules = new Set([
  'lib0',
  'y-protocols'
])
*/

const debugResolve = {
  resolveId (importee) {
    if (importee === 'y-codemirror.next') {
      return `${process.cwd()}/src/index.js`
    }
    if (importee === 'yjs/tests/testHelper.js') {
      return `${process.cwd()}/node_modules/yjs/tests/testHelper.js`
    }
    if (importee === 'yjs') {
      return `${process.cwd()}/node_modules/yjs/src/index.js`
    }
    /*
    if (customModules.has(importee.split('/')[0])) {
      return `${process.cwd()}/../${importee}/src/${importee}.js`
    }
    if (customLibModules.has(importee.split('/')[0])) {
      return `${process.cwd()}/../${importee}`
    }
    */
    return null
  }
}

const minificationPlugins = process.env.PRODUCTION != null
  ? [
    terser({
      module: true,
      compress: {
        hoist_vars: true,
        module: true,
        passes: 1,
        pure_getters: true,
        unsafe_comps: true,
        unsafe_undefined: true
      },
      mangle: {
        toplevel: true
      }
    })
  ] : []

export default [{
  input: './src/index.js',
  external: id => /^(lib0|yjs|y-protocols|simple-peer)/.test(id),
  output: [{
    name: 'y-codemirror',
    file: 'dist/y-codemirror.cjs',
    format: 'cjs',
    sourcemap: true,
    paths: path => {
      if (/^lib0\//.test(path)) {
        return `lib0/dist${path.slice(4)}.cjs`
      } else if (/^y-protocols\//.test(path)) {
        return `y-protocols/dist${path.slice(11)}.cjs`
      }
      return path
    }
  }]
}, {
  input: './demo/codemirror.js',
  output: {
    name: 'test',
    file: 'dist/demo.js',
    format: 'iife',
    sourcemap: true
  },
  plugins: [
    debugResolve,
    nodeResolve({
      mainFields: ['module', 'browser', 'main']
    }),
    commonjs(),
    ...minificationPlugins
  ]
}, {
  input: './test/index.js',
  output: {
    name: 'test',
    file: 'dist/test.js',
    format: 'iife',
    sourcemap: true
  },
  plugins: [
    debugResolve,
    nodeResolve({
      mainFields: ['module', 'browser', 'main']
    }),
    commonjs()
  ]
}, {
  input: './test/index.js',
  output: {
    name: 'test',
    file: 'dist/test.cjs',
    format: 'cjs',
    sourcemap: true,
    paths: path => {
      if (/^lib0\//.test(path)) {
        return `lib0/dist${path.slice(4)}.cjs`
      }
    }
  },
  plugins: [
    debugResolve,
    nodeResolve({
      mainFields: ['module', 'main']
    })
  ],
  external: id => /^(lib0|fs|path|jsdom|isomorphic)/.test(id)
}]
