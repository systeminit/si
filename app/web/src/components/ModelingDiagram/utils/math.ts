import { Vector2d, IRect } from "konva/lib/types";

/** return distance between 2 points */
export function vectorDistance(v1: Vector2d, v2: Vector2d) {
  return Math.sqrt((v1.x - v2.x) ** 2 + (v1.y - v2.y) ** 2);
}

/** add 2 vectors */
export function vectorAdd(v1: Vector2d, v2: Vector2d) {
  return {
    x: v1.x + v2.x,
    y: v1.y + v2.y,
  } as Vector2d;
}

export function vectorBetween(v1: Vector2d, v2: Vector2d) {
  return {
    x: v2.x - v1.x,
    y: v2.y - v1.y,
  } as Vector2d;
}

export function pointsToRect(v1: Vector2d, v2: Vector2d) {
  return {
    x: v1.x,
    y: v1.y,
    width: v2.x - v1.x,
    height: v2.y - v1.y,
  };
}

/** check if 2 rectangles overlap, each rect defined by 2 opposite corner points */
export function checkRectanglesOverlap(rect1: IRect, rect2: IRect) {
  const r1v1: Vector2d = { x: rect1.x, y: rect1.y };
  const r1v2: Vector2d = {
    x: rect1.x + rect1.width,
    y: rect1.y + rect1.height,
  };
  const r2v1: Vector2d = { x: rect2.x, y: rect2.y };
  const r2v2: Vector2d = {
    x: rect2.x + rect2.width,
    y: rect2.y + rect2.height,
  };

  // we allow any 2 points to be passed in, but we need the points in a sorted order for this algorithm
  // so we use min/max to make sure we're getting the bottom left and top right of each rect
  const r1x1 = Math.min(r1v1.x, r1v2.x);
  const r1y1 = Math.min(r1v1.y, r1v2.y);
  const r1x2 = Math.max(r1v1.x, r1v2.x);
  const r1y2 = Math.max(r1v1.y, r1v2.y);
  const r2x1 = Math.min(r2v1.x, r2v2.x);
  const r2y1 = Math.min(r2v1.y, r2v2.y);
  const r2x2 = Math.max(r2v1.x, r2v2.x);
  const r2y2 = Math.max(r2v1.y, r2v2.y);

  // check if either rect is fully outside the other
  // by checking left edge vs right edge
  if (r1x1 > r2x2 || r2x1 > r1x2) return false;
  // then top edge vs bottom edge
  if (r1y1 > r2y2 || r2y1 > r1y2) return false;
  // otherwise they must be overlapping
  return true;
}

/**
 * returns a new point at a set px distance along a line from P1 to P2
 */
export function pointAlongLinePx(
  p1: Vector2d,
  p2: Vector2d,
  pxDistanceFromP1: number,
): Vector2d {
  const distance = vectorDistance(p1, p2);
  const pctDistance = pxDistanceFromP1 / distance;
  return pointAlongLinePct(p1, p2, pctDistance);
}

/**
 * returns a new point at a set % distance along a line from P1 to P2
 */
export function pointAlongLinePct(
  p1: Vector2d,
  p2: Vector2d,
  pctDistanceFromP1: number,
): Vector2d {
  return {
    x: p1.x + pctDistanceFromP1 * (p2.x - p1.x),
    y: p1.y + pctDistanceFromP1 * (p2.y - p1.y),
  };
}

/**
 * Returns whether a rect contains another one
 */
export function rectContainsAnother(container: IRect, object: IRect) {
  const insideX =
    object.x >= container.x &&
    object.x + object.width <= container.x + container.width;

  const insideY =
    object.y >= container.y &&
    object.y + object.height <= container.y + container.height;

  return insideX && insideY;
}

export function rectContainsPoint(rect: IRect, point: Vector2d) {
  return (
    point.x >= rect.x &&
    point.x <= rect.x + rect.width &&
    point.y >= rect.y &&
    point.y <= rect.y + rect.height
  );
}

export function getRectCenter(rect: IRect) {
  return { x: rect.x + rect.width / 2, y: rect.y + rect.height / 2 };
}

export function shrinkRect(rect: IRect, shrinkPx: number) {
  return {
    x: rect.x + shrinkPx,
    y: rect.y + shrinkPx,
    width: rect.width - shrinkPx * 2,
    height: rect.height - shrinkPx * 2,
  };
}

export function getAdjustmentRectToContainAnother(
  container: IRect,
  object: IRect,
  paddingPx = 0,
) {
  const cMinX = container.x;
  const cMaxX = container.x + container.width;
  const cMinY = container.y;
  const cMaxY = container.y + container.height;

  const oMinX = object.x;
  const oMaxX = object.x + object.width;
  const oMinY = object.y;
  const oMaxY = object.y + object.height;

  let moveX = 0;
  let moveY = 0;

  if (oMinX < cMinX) {
    moveX = oMinX - cMinX - paddingPx;
  } else if (oMaxX > cMaxX) {
    moveX = oMaxX - cMaxX + paddingPx;
  }

  if (oMinY < cMinY) {
    moveY = oMinY - cMinY - paddingPx;
  } else if (oMaxY > cMaxY) {
    moveY = oMaxY - cMaxY + paddingPx;
  }

  return {
    x: moveX,
    y: moveY,
  };
}
