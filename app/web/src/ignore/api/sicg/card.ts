import {
  Cg2dCoordinate,
  CgSize,
  CgRectangle,
  CgScaleFactor,
  cgRectangleRelativeCenter,
} from "./cg";

export interface Card2d {
  face: CgRectangle;
  size: CgSize | undefined;
  center: Cg2dCoordinate | undefined;
  position: Cg2dCoordinate;
  scaleFactor: CgScaleFactor | undefined;
}

export class CgCard implements Card2d {
  face: Card2d["face"];
  size: Card2d["size"];
  center: Card2d["center"];
  position: Card2d["position"];
  scaleFactor: Card2d["scaleFactor"];

  constructor(rectangle: CgRectangle, position: Cg2dCoordinate) {
    this.face = rectangle;
    this.calculateSize();
    this.calculateCenter();
    this.position = position;
    this.scaleFactor = 1.0;
  }

  calculateSize() {
    // Naive way to calculate width and height, assuming the origin is topLeft
    this.size = {
      width:
        this.face.topRightVertex.position.x -
        this.face.topLeftVertex.position.x,
      height:
        this.face.bottomLeftVertex.position.y -
        this.face.topLeftVertex.position.y,
    } as CgSize;
  }

  calculateCenter() {
    this.center = cgRectangleRelativeCenter(this.face);
  }

  setScaleFactor(scaleFactor: CgScaleFactor) {
    this.scaleFactor = scaleFactor;
  }
}
