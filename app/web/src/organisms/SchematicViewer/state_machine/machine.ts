import { createMachine } from "xstate";

import { PanEventKind, PanEvent, PanState } from "./pan";
import { DragEventKind, DragEvent, DragState } from "./drag";
import { SelectEventKind, SelectEvent, SelectState } from "./select";
import { ZoomEventKind, ZoomEvent, ZoomState } from "./zoom";
import { NodeAddEventKind, NodeAddEvent, NodeAddState } from "./node_add";

import { ConnectEventKind, ConnectEvent, ConnectState } from "./connect";

import { IdleState } from "./idle";

export const ViewerEventKind = {
  ...ZoomEventKind,
  ...ConnectEventKind,
  ...DragEventKind,
  ...PanEventKind,
  ...SelectEventKind,
  ...NodeAddEventKind,
};
export type ViewerEvent =
  | PanEvent
  | DragEvent
  | SelectEvent
  | ConnectEvent
  | ZoomEvent
  | NodeAddEvent;

export const ViewerState = {
  ...ZoomState,
  ...SelectState,
  ...ConnectState,
  ...DragState,
  ...PanState,
  ...IdleState,
  ...NodeAddState,
};

/**
 * Idle state machine implementation
 */
const IdleMachineState = {
  [IdleState.IDLING]: {
    on: {
      [PanEventKind.ACTIVATE_PANNING]: {
        target: PanState.PANNING_ACTIVATED,
      },
      [DragEventKind.ACTIVATE_DRAGGING]: {
        target: DragState.DRAGGING_ACTIVATED,
      },
      [SelectEventKind.ACTIVATE_SELECTING]: {
        target: SelectState.SELECTING_ACTIVATED,
      },
      [ConnectEventKind.ACTIVATE_CONNECTING]: {
        target: ConnectState.CONNECTING_ACTIVATED,
      },
      [ZoomEventKind.ACTIVATE_ZOOMING]: {
        target: ZoomState.ZOOMING_ACTIVATED,
      },
      [NodeAddEventKind.ACTIVATE_NODEADD]: {
        target: NodeAddState.NODEADD_ACTIVATED,
      },
    },
  },
};

/**
 * Pan state machine implementation
 */
const PanMachineState = {
  [PanState.PANNING_ACTIVATED]: {
    on: {
      [PanEventKind.INITIATE_PANNING]: {
        target: PanState.PANNING_INITIATED,
      },
      [PanEventKind.DEACTIVATE_PANNING]: {
        target: IdleState.IDLING,
      },
    },
  },
  [PanState.PANNING_INITIATED]: {
    on: {
      [PanEventKind.DEACTIVATE_PANNING]: {
        target: IdleState.IDLING,
      },
      [PanEventKind.PANNING]: {
        target: PanState.PANNING,
      },
    },
  },
  [PanState.PANNING]: {
    on: {
      [PanEventKind.DEACTIVATE_PANNING]: {
        target: IdleState.IDLING,
      },
    },
  },
};

/**
 * Zoom state machine implementation
 */
const ZoomMachineState = {
  [ZoomState.ZOOMING_ACTIVATED]: {
    on: {
      [ZoomEventKind.INITIATE_ZOOMING]: {
        target: ZoomState.ZOOMING_INITIATED,
      },
      [ZoomEventKind.DEACTIVATE_ZOOMING]: {
        target: IdleState.IDLING,
      },
    },
  },
  [ZoomState.ZOOMING_INITIATED]: {
    on: {
      [ZoomEventKind.DEACTIVATE_ZOOMING]: {
        target: IdleState.IDLING,
      },
      [ZoomEventKind.ZOOMING]: {
        target: ZoomState.ZOOMING,
      },
    },
  },
  [ZoomState.ZOOMING]: {
    on: {
      [ZoomEventKind.DEACTIVATE_ZOOMING]: {
        target: IdleState.IDLING,
      },
    },
  },
};

/**
 * Drag state machine implementation
 */
const DragMachineState = {
  [DragState.DRAGGING_ACTIVATED]: {
    on: {
      [DragEventKind.INITIATE_DRAGGING]: {
        target: DragState.DRAGGING_INITIATED,
      },
      [DragEventKind.DEACTIVATE_DRAGGING]: {
        target: IdleState.IDLING,
      },
    },
  },
  [DragState.DRAGGING_INITIATED]: {
    on: {
      [DragEventKind.DEACTIVATE_DRAGGING]: {
        target: IdleState.IDLING,
      },
      [DragEventKind.DRAGGING]: {
        target: DragState.DRAGGING,
      },
    },
  },
  [DragState.DRAGGING]: {
    on: {
      [DragEventKind.DEACTIVATE_DRAGGING]: {
        target: IdleState.IDLING,
      },
    },
  },
};

/**
 * Connect state machine implementation
 */
const ConnectMachineState = {
  [ConnectState.CONNECTING_ACTIVATED]: {
    on: {
      [ConnectEventKind.INITIATE_CONNECTING]: {
        target: ConnectState.CONNECTING_INITIATED,
      },
      [ConnectEventKind.DEACTIVATE_CONNECTING]: {
        target: IdleState.IDLING,
      },
    },
  },
  [ConnectState.CONNECTING_INITIATED]: {
    on: {
      [ConnectEventKind.DEACTIVATE_CONNECTING]: {
        target: IdleState.IDLING,
      },
      [ConnectEventKind.CONNECTING]: {
        target: ConnectState.CONNECTING,
      },
    },
  },
  [ConnectState.CONNECTING]: {
    on: {
      [ConnectEventKind.CONNECTING_TO_SOCKET]: {
        target: ConnectState.CONNECTING_TO_SOCKET,
      },
      [ConnectEventKind.DEACTIVATE_CONNECTING]: {
        target: IdleState.IDLING,
      },
    },
  },
  [ConnectState.CONNECTING_TO_SOCKET]: {
    on: {
      [ConnectEventKind.DEACTIVATE_CONNECTING]: {
        target: IdleState.IDLING,
      },
    },
  },
};

/**
 * Select state machine implementation
 */
const SelectMachineState = {
  [SelectState.SELECTING_ACTIVATED]: {
    on: {
      [SelectEventKind.INITIATE_SELECTING]: {
        target: SelectState.SELECTING_INITIATED,
      },
      [SelectEventKind.DEACTIVATE_SELECTING]: {
        target: IdleState.IDLING,
      },
    },
  },
  [SelectState.SELECTING_INITIATED]: {
    on: {
      [SelectEventKind.DEACTIVATE_SELECTING]: {
        target: IdleState.IDLING,
      },
      [SelectEventKind.SELECTING]: {
        target: SelectState.SELECTING,
      },
      [SelectEventKind.DESELECTING]: {
        target: SelectState.DESELECTING,
      },
    },
  },
  [SelectState.SELECTING]: {
    on: {
      [SelectEventKind.DEACTIVATE_SELECTING]: {
        target: IdleState.IDLING,
      },
      [DragEventKind.ACTIVATE_DRAGGING]: {
        target: DragState.DRAGGING_ACTIVATED,
      },
    },
  },
  [SelectState.DESELECTING]: {
    on: {
      [SelectEventKind.DEACTIVATE_SELECTING]: {
        target: IdleState.IDLING,
      },
    },
  },
};

/**
 * Node add state machine implementation
 */
const NodeAddMachineState = {
  [NodeAddState.NODEADD_ACTIVATED]: {
    on: {
      [NodeAddEventKind.INITIATE_NODEADD]: {
        target: NodeAddState.NODEADD_INITIATED,
      },
      [NodeAddEventKind.DEACTIVATE_NODEADD]: {
        target: IdleState.IDLING,
      },
    },
  },
  [NodeAddState.NODEADD_INITIATED]: {
    on: {
      [NodeAddEventKind.DEACTIVATE_NODEADD]: {
        target: IdleState.IDLING,
      },
      [NodeAddEventKind.ADDING_NODE]: {
        target: NodeAddState.ADDING_NODE,
      },
    },
  },
  [NodeAddState.ADDING_NODE]: {
    on: {
      [NodeAddEventKind.DEACTIVATE_NODEADD]: {
        target: IdleState.IDLING,
      },
    },
  },
};

/**
 * Viewer state machine
 */
export class ViewerStateMachine {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  machine: any;

  constructor() {
    this.machine = createMachine<ViewerEvent>({
      key: "viewer",
      initial: IdleState.IDLING,
      states: {
        ...IdleMachineState,
        ...PanMachineState,
        ...ZoomMachineState,
        ...DragMachineState,
        ...ConnectMachineState,
        ...SelectMachineState,
        ...NodeAddMachineState,
      },
    });
  }
}
