export const RESIZER_SIZE = 4;

export enum PanelType {
  Attribute = "attribute",
  Secret = "secret",
  Schematic = "schematic",
  Empty = "empty",
}

export interface PanelMaximized {
  panelIndex: number;
  panelRef: string;
  panelContainerRef: string;
}

export interface ResizeEvent {
  panelCheech: {
    panelIndex: number;
    size: Size;
  };
  panelChong: {
    panelIndex: number;
    size: Size;
  };
  startingPosition: {
    clientX: number;
    clientY: number;
  };
}

export interface Size {
  heightPx: number;
  widthPx: number;
}

export interface IPanel {
  name: string;
  type: "panel";
}

export type IPanelOrPanelContainer = IPanel | IPanelContainer;

export type IPanelContainer =
  | IPanelContainerWithoutSize
  | IPanelContainerWithSize;

export interface IPanelContainerWithoutSize {
  orientation: "column" | "row";
  panels: IPanelOrPanelContainer[];
  type: "panelContainer";
  width?: never;
}

export interface IPanelContainerWithSize {
  orientation: "column" | "row";
  panels: IPanelOrPanelContainer[];
  type: "panelContainer";
  width: number;
}
