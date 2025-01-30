//import { makeConsole } from "./console.ts";
import {
  GROUP_HEADER_BOTTOM_MARGIN,
  MIN_NODE_DIMENSION,
  NODE_HEADER_HEIGHT,
  NODE_PADDING_BOTTOM,
  NODE_SUBTITLE_TEXT_HEIGHT,
  SOCKET_GAP,
  SOCKET_MARGIN_TOP,
  SOCKET_SIZE,
} from "./diagram_constants.ts";
// Simple row layout
// Takes in Components, who have a fixed height and width
// Takes in Frames, which have a flexible height and width
// Frames have Rows, and components can be added to them
// All components must start from a frame

export enum LayoutKind {
  Component = "COMPONENT",
  Frame = "FRAME",
  Row = "ROW",
}

export interface Component {
  kind: LayoutKind.Component;
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Row {
  items: Item[];
  id: string;
  kind: LayoutKind.Row;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Frame {
  kind: LayoutKind.Frame;
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
  rows: Row[];
}

type Item = Frame | Component;

const WIDTH_MARGIN = 50;
const HEIGHT_MARGIN = 50;
const FRAME_LABEL_HEIGHT = NODE_HEADER_HEIGHT + GROUP_HEADER_BOTTOM_MARGIN;

export default {
  initialFrame,
  createFrame,
  createComponent,
  addToRow,
  calculateSizeAndPosition,
  createRow,
  setGeometryForSpecs,
};

// Creates the first frame; sets the X and Y automatically.
export function initialFrame(frameId: string, x = 0, y = 500): Frame {
  const frame = createFrame(frameId);
  frame.x = x;
  frame.y = y;
  return frame;
}

// Create a new frame
export function createFrame(frameId: string): Frame {
  return {
    id: frameId,
    kind: LayoutKind.Frame,
    x: 0,
    y: 0,
    width: 0,
    height: 0,
    rows: [],
  };
}

function componentHeight(numSockets: number) {
  // PSA: This is duplicated in app web layout code. Change in both places!
  return (
    NODE_SUBTITLE_TEXT_HEIGHT +
    SOCKET_MARGIN_TOP +
    SOCKET_GAP *
      (numSockets -
        1) +
    SOCKET_SIZE / 2 +
    // TODO: this isn't right yet!
    NODE_PADDING_BOTTOM +
    // (statusIcons?.value.length ? 30 : 0)
    30 + // keeping this there as a constant for the moment
    NODE_HEADER_HEIGHT
  );
}

// Create a new component
export function createComponent(
  componentId: string,
  numSockets: number,
): Component {
  return {
    kind: LayoutKind.Component,
    id: componentId,
    x: 0,
    y: 0,
    width: MIN_NODE_DIMENSION,
    height: componentHeight(numSockets),
  };
}

// Create a new row; prefer just calling addtoRow()
export function createRow(rowId: string): Row {
  return {
    items: [],
    id: rowId,
    kind: LayoutKind.Row,
    x: 0,
    y: 0,
    width: 0,
    height: 0,
  };
}

// Adds an item to the specified frame and rowId; creating
// the row if it does not exist. The row will be automatically
// positioned in the order it is created.
export function addToRow(frame: Frame, rowId: string, item: Item) {
  // Check to see if a row with this ID exists
  // If it does not, a new Row is created and the item pushed to the end
  // Otherwise, push to the end

  const row = frame.rows.find((r) => r.id === rowId);
  if (row) {
    const exists = row.items.find((i) => i.id === item.id);
    if (!exists) {
      row.items.push(item);
    }
  } else {
    const row = createRow(rowId);
    row.items.push(item);
    frame.rows.push(row);
  }
}

export function calculateSizeAndPosition(fromFrame: Frame): Frame {
  // const console = makeConsole("poop");
  const framesToCheck = [fromFrame];
  const nextFrames = [fromFrame];
  // Control against infinite loop
  const maxChecks = 10;
  let checkCount = 0;

  // Collect all the frames
  while (nextFrames.length) {
    if (checkCount > maxChecks) {
      throw new Error(
        "Exceeded maximum number of frame recursions; probably a loop in there?",
      );
    }
    for (const row of nextFrames[0].rows) {
      for (const item of row.items) {
        if (item.kind === LayoutKind.Frame) {
          framesToCheck.push(item);
          nextFrames.push(item);
        }
      }
    }
    checkCount++;
    nextFrames.shift();
  }

  // Reverse their order, so we are walking from the bottom up
  framesToCheck.reverse();

  // Now, calculate the height and width of every frame
  for (const frame of framesToCheck) {
    // Calculate the height and width of every row
    for (const row of frame.rows) {
      let hasSubFrame = false;
      for (const [idx, item] of row.items.entries()) {
        if (item.kind === LayoutKind.Frame) {
          hasSubFrame = true;
        }
        if (item.height > row.height) {
          row.height = item.height;
        }
        if (idx === row.items.length - 1 && row.items.length !== 1) {
          row.width += item.width;
        } else {
          row.width = row.width + item.width + WIDTH_MARGIN;
        }
      }
      if (row.width > frame.width) {
        frame.width = row.width;
      }
      if (hasSubFrame) {
        frame.height = frame.height + row.height + HEIGHT_MARGIN +
          FRAME_LABEL_HEIGHT;
      } else {
        frame.height = frame.height + row.height + HEIGHT_MARGIN;
      }
      // adding defaults to prevent 0x0 rendering attempts
      if (!frame.height) frame.height = 500;
      if (!frame.width) frame.width = 500;
    }
  }

  // The 0 position for X is actually (0 - center) of the root frame; halfway from the center, you would imagine
  const componentPadding = MIN_NODE_DIMENSION / 2; // half the width of a component plus padding on each side

  // Now, calculate the X and Y of everything from the top down, rather than bottom up
  // NOTE: This is mid-refactor to add frames nested in frames, which likely
  // makes the layout wrong
  framesToCheck.reverse();
  for (const frame of framesToCheck) {
    const computedRowY = frame.y;
    const computedRowX = (frame.x - (frame.width / 2)) + componentPadding;

    let lastRowY = 0;
    for (const row of frame.rows) {
      row.x = computedRowX;
      row.y = computedRowY + lastRowY;
      let lastItemX = row.x;
      let hasFrame = false;
      for (const item of row.items) {
        if (item.kind === LayoutKind.Frame) {
          hasFrame = true;
          item.y = row.y + (HEIGHT_MARGIN / 2) + FRAME_LABEL_HEIGHT;
        } else {
          item.y = row.y + (HEIGHT_MARGIN / 2);
        }
        item.x = lastItemX + (WIDTH_MARGIN / 2);
        lastItemX = item.x + item.width;
      }
      lastRowY += row.height + HEIGHT_MARGIN;
      if (hasFrame) lastRowY += NODE_HEADER_HEIGHT;
    }
  }

  return fromFrame;
}

export function setGeometryForSpecs(fromFrame: Frame, specs: any[]) {
  //const console = makeConsole("poop");
  const components: Item[] = [fromFrame];
  const nextCheck: Item[] = [fromFrame];
  // Control against infinite loop
  const maxChecks = 10;
  let checkCount = 0;

  // Collect all the frames
  while (nextCheck.length) {
    if (checkCount > maxChecks) {
      throw new Error(
        "Exceeded maximum number of frame recursions; probably a loop in there?",
      );
    }

    if (nextCheck[0].kind === LayoutKind.Frame) {
      for (const row of nextCheck[0].rows) {
        for (const item of row.items) {
          components.push(item);
          if (item.kind === LayoutKind.Frame) {
            nextCheck.push(item);
          }
        }
      }
    }

    checkCount++;
    nextCheck.shift();
  }

  for (const spec of specs) {
    const component = components.find((c) => c.id === spec.properties.si.name);
    if (component) {
      const geo = {
        x: component.x,
        y: component.y,
        width: component.width,
        height: component.height,
      };
      spec.geometry = geo;
    }
  }
}
