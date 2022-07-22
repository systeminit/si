import { SchematicKind } from "@/api/sdf/dal/schematic";

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

export enum PanelAttributeSubType {
  Attributes = "attribute",
  Qualifications = "qualification",
}

export interface IPanelSchematic {
  name: PanelType.Schematic;
  type: "panel";
  subType: SchematicKind;
}

export interface IPanelAttribute {
  name: PanelType.Attribute;
  type: "panel";
  subType: PanelAttributeSubType;
}

export interface IPanel {
  name: PanelType;
  type: "panel";
  subType: never;
}

export type IPanelOrPanelContainer =
  | IPanel
  | IPanelAttribute
  | IPanelSchematic
  | IPanelContainer;

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
