import { Vector2d } from "konva/lib/types";

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

/** check if 2 rectangles overlap, each rect defined by 2 opposite corner points */
export function checkRectanglesOverlap(
  r1v1: Vector2d,
  r1v2: Vector2d,
  r2v1: Vector2d,
  r2v2: Vector2d,
) {
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
 * returns a new point at a set distance along a line from P1 to P2
 */
export function pointAlongLine(
  p1: Vector2d,
  p2: Vector2d,
  distanceFromP1: number,
): Vector2d {
  const distance = vectorDistance(p1, p2);
  return {
    x: p1.x + (distanceFromP1 / distance) * (p2.x - p1.x),
    y: p1.y + (distanceFromP1 / distance) * (p2.y - p1.y),
  };
}
