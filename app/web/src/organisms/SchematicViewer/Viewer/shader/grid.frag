#ifdef GL_ES
precision mediump float;
#endif

varying vec2 vUvs;

uniform vec3 uColor;
uniform float uBorderThickness;
uniform float uGridSubdivisions;
uniform float uZoomFactor;

float rect(in vec2 st, in float thickness) {
    vec2 bottomLeftCorner = step(vec2(thickness), st);
    vec2 topRightCorner = step(vec2(thickness), 1. - st);
    float border = bottomLeftCorner.s * bottomLeftCorner.t * topRightCorner.s * topRightCorner.t;
    return 1.0 - border;
}

void main() {
    float thicknessAdjustment = pow((1. / uZoomFactor), 1.2);

    vec2 st1 = vUvs;
    st1 *= uGridSubdivisions / uZoomFactor;
    st1 = fract(st1); // x - floor(x)
    float rectInner = rect(st1, uBorderThickness * 1.75 * thicknessAdjustment);

    float alpha = rectInner * uZoomFactor;

    gl_FragColor = vec4(uColor, alpha);
}
