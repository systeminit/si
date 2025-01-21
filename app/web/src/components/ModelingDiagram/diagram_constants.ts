import { COLOR_PALETTE } from "@si/vue-lib/design-system";

// how far a user must move the mouse after initial click to start "dragging"
export const DRAG_DISTANCE_THRESHOLD = 5;

// if dragging to the edge of the screen, within this area (pixels) will trigger scrolling in that direction
export const DRAG_EDGE_TRIGGER_SCROLL_WIDTH = 15;

// corner radius used on nodes (maybe other things later?)
export const CORNER_RADIUS = 3;

// default node color if nothing passed in
// TODO: this is that random purple color... check with mark
export const DEFAULT_NODE_COLOR = "#8B39CB";

// font family used for all text elements
export const DIAGRAM_FONT_FAMILY = "Inter";

// color used to show what is selected (currently a nice blue)
export const SELECTION_COLOR = COLOR_PALETTE.action[300];

export const MIN_ZOOM = 0.1; // 10%
export const MAX_ZOOM = 10; // 1000%

export const NODE_TITLE_HEADER_MARGIN_RIGHT = 56;

export const GROUP_TITLE_FONT_SIZE = 14;
export const GROUP_INTERNAL_PADDING = 20;
export const GROUP_RESIZE_HANDLE_SIZE = 20;
export const GROUP_HEADER_ICON_SIZE = 35;
// We need an extra bottom padding to account for the status icons
export const GROUP_BOTTOM_INTERNAL_PADDING = 35;
export const GROUP_DEFAULT_WIDTH = 35;
export const GROUP_DEFAULT_HEIGHT = 35;

// TODO (Wendy) - this constant should be derived from the frame header and internal padding
export const GROUP_INNER_Y_BOUNDARY_OFFSET = 59;

export const SOCKET_TOP_MARGIN = 11;
export const NODE_WIDTH = 200;

/**
 * NOTE: WE HAVE DUP'D THE BELOW CONSTANTS INTO LANG-JS
 * IF YOU CHANGE ANY HERE, GO CHANGE THOSE TO MATCH
 *
 * todo: stop making a human copy these, figure out either
 * a monorepo import, or a build step to copy
 * */

// Default Width for Node
export const MIN_NODE_DIMENSION = NODE_WIDTH + 20 * 2;
export const GROUP_HEADER_BOTTOM_MARGIN = 14;
export const NODE_HEADER_HEIGHT = 44;

export const NODE_HEADER_TEXT_HEIGHT = 12;
export const NODE_SUBTITLE_TEXT_HEIGHT = 11;

// spacing between sockets
export const SOCKET_GAP = 22;
// spacing above/below sockets within the node
export const SOCKET_MARGIN_TOP = 8; // less because there is also a subtitle with some padding
export const NODE_PADDING_BOTTOM = 10; //
// width/height of sockets
export const SOCKET_SIZE = 15;
