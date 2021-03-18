import {
  cgRectangleFromDomRect,
  cgRectangleRelativeCenter,
  Cg2dCoordinate,
  CgMousePosition,
} from "./cg";

export function cgSetElementPositionToViewportCenter(
  element: HTMLElement,
  viewport: HTMLElement,
) {
  let elementContainerSquare = cgRectangleFromDomRect(
    element.getBoundingClientRect(),
  );
  let elementContainerCenter = cgRectangleRelativeCenter(
    elementContainerSquare,
  );

  let viewportSquare = cgRectangleFromDomRect(viewport.getBoundingClientRect());
  let viewportCenter = cgRectangleRelativeCenter(viewportSquare);

  let offset = {
    x: viewportCenter.x - elementContainerCenter.x,
    y: viewportCenter.y - elementContainerCenter.y,
  };

  // This will override style on the element.
  // TODO: It should update instead of overriding.
  element.setAttribute(
    "style",
    "left:" + offset.x + "px;" + "top:" + offset.y + "px;",
  );
}

export function cgSetElementPosition(
  element: HTMLElement,
  position: Cg2dCoordinate,
): void {
  element.setAttribute(
    "style",
    "left:" + position.x + "px;" + "top:" + position.y + "px;",
  );
}

export function cgSetElementSize(
  element: HTMLElement,
  width: number,
  height: number,
) {
  element.setAttribute(
    "style",
    "width:" + width + "px;" + "height:" + height + "px;",
  );
}

export function cgGetMousePositionInElementSpace(
  mouseEvent: MouseEvent,
  element: HTMLElement,
): CgMousePosition {
  let elementRect = element.getBoundingClientRect();
  let mousePosition: CgMousePosition = {
    x: mouseEvent.clientX,
    y: mouseEvent.clientY,
  };

  let relativeMousePosition: CgMousePosition = {
    x: mousePosition.x - elementRect.left,
    y: mousePosition.y - elementRect.top,
  };
  return relativeMousePosition;
}
