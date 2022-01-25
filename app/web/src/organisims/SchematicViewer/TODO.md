# Todo

## Refactor

- Scene Management
- Interaction Management
- Data Management
- State Management
- Rendering

## Improvements

- ZoomFactor should always be dynamic. If the user scrolls while dragging, the zoomFactor should update.
- Grid should be "infinite".
- Spacebar should maximize/minimize panel.
- Selected node should stay selected until the selection is cleared.
- Socket labels should not be interactive, so that they don't occlude the node when clicking on a node. (the socket object should be a container.)
- Implement Interpreter type in fsms (will resolve all any warnings)
- implement siCtx to store si metadata on webGl objs.
- Implement node reactiveness through the data manager.

## Look and feel

- background grid color
- background grid size
- background grid (SVG was better, should consider reverting to using an SVG instead of webGL)
  > SVG is sharper, and renders much better.
