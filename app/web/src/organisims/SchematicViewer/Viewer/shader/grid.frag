#ifdef GL_ES
precision mediump float;
#endif

varying vec2 vUvs;

uniform vec3 uColor;
uniform float uBorderThickness;
uniform float uGridSubdivisions;

float rect(in vec2 _st, in float _thickness) {
    vec2 bottomLeftCorner = step(vec2(_thickness), _st);
    vec2 topRightCorner = step(vec2(_thickness), 1.0 - _st);
    float border = bottomLeftCorner.s * bottomLeftCorner.t * topRightCorner.s * topRightCorner.t;
    return 1.0 - border;
}

void main() {
    vec3 color = vec3(uColor);
    vec2 st1 = vUvs;
    vec2 st2 = vUvs;

    //  st1
    st1 *= uGridSubdivisions;
    st1 = fract(st1); // x - floor(x)
    float rectInner = rect(st1, uBorderThickness);
    float alphaInner = rectInner;


    //  st2
    st2 *= ( uGridSubdivisions * 0.25);
    st2 = fract(st2); // x - floor(x)
    float rectOuter = rect(st2, uBorderThickness);
    float alphaOuter = rectOuter;

    float alpha = max(rectInner * 0.75, rectOuter);

    gl_FragColor = vec4(color, alpha);
}