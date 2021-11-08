export { CgResolution } from "@/api/sicg/cg";
export { CgColorRgb } from "@/api/sicg/cg";
export { CgColorHex } from "@/api/sicg/cg";
export { Cg2dCoordinate } from "@/api/sicg/cg";
export { Cg2dVertex } from "@/api/sicg/cg";
export { CgRectangle } from "@/api/sicg/cg";
export { CgScaleFactor } from "@/api/sicg/cg";
export { CgSize } from "@/api/sicg/cg";
export { CgMousePosition } from "@/api/sicg/cg";
export { CgCard } from "@/api/sicg/card";

import { cgVectorRelativeCenter } from "@/api/sicg/cg";
import { cgRectangleFromDomRect } from "@/api/sicg/cg";
import { cgRectangleRelativeCenter } from "@/api/sicg/cg";
import { cgSetElementPositionToViewportCenter } from "@/api/sicg/element";
import { cgSetElementSize } from "@/api/sicg/element";
import { cgSetElementPosition } from "@/api/sicg/element";
import { cgGetMousePositionInElementSpace } from "@/api/sicg/element";

export const SiCg = {
  cgVectorRelativeCenter,
  cgRectangleFromDomRect,
  cgRectangleRelativeCenter,
  cgSetElementPositionToViewportCenter,
  cgSetElementSize,
  cgSetElementPosition,
  cgGetMousePositionInElementSpace,
};
