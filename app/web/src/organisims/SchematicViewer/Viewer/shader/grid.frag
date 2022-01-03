#ifdef GL_ES
precision mediump float;
#endif

varying vec2 vUvs;

uniform vec3 uColor;
uniform float uBorderThickness;
uniform float uGridSubdivisions;
uniform float uZoomFactor;

float rect(in vec2 _st, in float _thickness) {
    vec2 bottomLeftCorner = step(vec2(_thickness), _st);
    vec2 topRightCorner = step(vec2(_thickness), 1.0 - _st);
    float border = bottomLeftCorner.s * bottomLeftCorner.t * topRightCorner.s * topRightCorner.t;
    return 1.0 - border;
}

void main() {
    vec3 color = vec3(uColor);
    vec2 st1 = vUvs;

    float thicknessAdjustment = pow((1.0 / uZoomFactor), 1.2);
    st1 *= uGridSubdivisions;
    st1 = fract(st1); // x - floor(x)
    float rectInner = rect(st1, uBorderThickness * 1.75 * thicknessAdjustment);

    float alpha = rectInner * clamp(uZoomFactor, 0.1, 1.0);

    gl_FragColor = vec4(color, alpha);
}