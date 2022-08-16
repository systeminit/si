## Diagram notes

### Konva gotchas + tips

- Heirarchy of components is Stage > Layer > everything else
- each Layer is a new Canvas in the dom, so only use a few layers
- cannot `v-if` on a Layer
- z-index is based on order items are added to the stage, so
  - reorder the underlying data to reorder an array of items
  - sometimes you may need to toggle `visible` rather than use v-if so the item stays at the correct level
- vue-konva components can be configured using the `config` prop or via individual prop bindings, however those only work with `camelCase` prop names, so we'll stick with config objects
- add `listening: false` on anything that does not need pointer events. this will help with perf!
