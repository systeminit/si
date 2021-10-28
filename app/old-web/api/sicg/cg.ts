export type shortcutState = {
  spacebarPressed: boolean;
  mouseMoved: boolean;
};

export type CgMousePosition = Cg2dCoordinate;

export interface CgColorRgb {
  red: number;
  green: number;
  blue: number;
}

export interface CgColorHex {
  color: string;
}

export interface CgResolution {
  x: number;
  y: number;
}

export type CgScaleFactor = number;

export interface CgSize {
  width: number;
  height: number;
}

export interface Cg2dCoordinate {
  x: number;
  y: number;
}

export interface Cg2dVertex {
  position: Cg2dCoordinate;
}

export interface CgRectangle {
  topLeftVertex: Cg2dVertex;
  topRightVertex: Cg2dVertex;
  bottomRightVertex: Cg2dVertex;
  bottomLeftVertex: Cg2dVertex;
}

// Calculates the mid po
export function cgVectorRelativeCenter(
  pt1: Cg2dCoordinate,
  pt2: Cg2dCoordinate,
): Cg2dCoordinate {
  let middle = 0.5;
  let midPoint: Cg2dCoordinate = {
    x: (pt2.x - pt1.x) * middle,
    y: (pt2.y - pt1.y) * middle,
  };
  return midPoint;
}

export function cgRectangleRelativeCenter(
  rectangle: CgRectangle,
): Cg2dCoordinate {
  let centerPoint = cgVectorRelativeCenter(
    rectangle.topLeftVertex.position,
    rectangle.bottomRightVertex.position,
  );
  return centerPoint;
}

export function cgRectangleFromDomRect(rect: DOMRect): CgRectangle {
  let rectangle: CgRectangle = {
    topLeftVertex: {
      position: {
        x: rect.left,
        y: rect.top,
      },
    } as Cg2dVertex,
    topRightVertex: {
      position: {
        x: rect.right,
        y: rect.top,
      },
    } as Cg2dVertex,
    bottomRightVertex: {
      position: {
        x: rect.right,
        y: rect.bottom,
      },
    } as Cg2dVertex,
    bottomLeftVertex: {
      position: {
        x: rect.left,
        y: rect.bottom,
      },
    } as Cg2dVertex,
  };
  return rectangle;
}
