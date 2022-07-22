precision mediump float;

/// Position of each corner of the grid
/// Should represent the canvas' rectangle
attribute vec2 aVertexPosition;

attribute vec2 aUvs;

varying vec2 vUvs;

/// Translates position to follow the camera
uniform mat3 translationMatrix;

/// Projects grid onto 2D plane
uniform mat3 projectionMatrix;

uniform float uGridSubdivisions;
uniform float uZoomFactor;

/// fmod implementation, pretty much `a % b`, but for floats
float modulo(float f1, float f2) {
  return f1 - (f2 * floor(f1 / f2));
}

void main() {
  vUvs = aUvs;

  vec3 camera = translationMatrix * vec3(1., 1., 1.);

  // We multiply squareSize by 10. to avoid the feeling of glitchy teleportation
  float squareSize = abs(aVertexPosition.x) / uGridSubdivisions * uZoomFactor * 10.;

  // Caps translation difference at 1 squareSize
  // Rendering a fixed grid, shifting it by at most 1 square
  // As it's symmetric
  vec3 pos = vec3(
    modulo(camera.x, squareSize) + aVertexPosition.x,
    modulo(camera.y, squareSize) + aVertexPosition.y,
    camera.z
  );

  // Sets point of one grid vertex
  gl_Position = vec4((projectionMatrix * pos).xy, 0.0, 1.0);
}
