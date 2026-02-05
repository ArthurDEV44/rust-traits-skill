# Raymarching & Signed Distance Functions (SDFs)

## Table of Contents
- [Raymarching Algorithm](#raymarching-algorithm)
- [SDF Primitives](#sdf-primitives)
- [SDF Boolean Operations](#sdf-boolean-operations)
- [Smooth Blending](#smooth-blending)
- [SDF Transformations](#sdf-transformations)
- [Normal Calculation](#normal-calculation)
- [Lighting Models](#lighting-models)
- [Shadows & AO](#shadows--ao)
- [Full Scene Example](#full-scene-example)

## Raymarching Algorithm

Cast rays from camera, step along ray by distance to nearest surface (SDF value), stop when close enough.

```glsl
float raymarch(vec3 ro, vec3 rd) {
  float t = 0.0;
  for (int i = 0; i < 100; i++) {
    vec3 p = ro + rd * t;
    float d = sceneSDF(p);
    if (d < 0.001) break;  // hit surface
    if (t > 100.0) break;  // too far
    t += d;
  }
  return t;
}
```

Setup in fragment shader (fullscreen quad):
```glsl
void main() {
  vec2 uv = (gl_FragCoord.xy - 0.5 * uResolution.xy) / uResolution.y;
  vec3 ro = vec3(0.0, 1.0, -3.0);           // ray origin (camera)
  vec3 rd = normalize(vec3(uv, 1.0));         // ray direction
  float t = raymarch(ro, rd);
  vec3 p = ro + rd * t;
  // shade the hit point...
}
```

## SDF Primitives

Each returns signed distance: negative inside, positive outside, zero on surface.

```glsl
// Sphere
float sdSphere(vec3 p, float r) {
  return length(p) - r;
}

// Box
float sdBox(vec3 p, vec3 b) {
  vec3 q = abs(p) - b;
  return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// Rounded box
float sdRoundBox(vec3 p, vec3 b, float r) {
  vec3 q = abs(p) - b;
  return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0) - r;
}

// Plane (y = 0)
float sdPlane(vec3 p) {
  return p.y;
}

// Torus
float sdTorus(vec3 p, vec2 t) {
  vec2 q = vec2(length(p.xz) - t.x, p.y);
  return length(q) - t.y;
}

// Cylinder
float sdCylinder(vec3 p, float h, float r) {
  vec2 d = abs(vec2(length(p.xz), p.y)) - vec2(r, h);
  return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
}

// Capsule (line segment)
float sdCapsule(vec3 p, vec3 a, vec3 b, float r) {
  vec3 pa = p - a, ba = b - a;
  float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
  return length(pa - ba * h) - r;
}
```

Full reference: [Inigo Quilez SDF functions](https://iquilezles.org/articles/distfunctions/)

## SDF Boolean Operations

```glsl
// Union: combine shapes
float opUnion(float d1, float d2) {
  return min(d1, d2);
}

// Subtraction: carve d2 from d1
float opSubtraction(float d1, float d2) {
  return max(d1, -d2);
}

// Intersection: keep overlap only
float opIntersection(float d1, float d2) {
  return max(d1, d2);
}
```

## Smooth Blending

Blend shapes with continuous transitions using smooth minimum.

```glsl
// Smooth union (k controls blend radius)
float opSmoothUnion(float d1, float d2, float k) {
  float h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
  return mix(d2, d1, h) - k * h * (1.0 - h);
}

// Smooth subtraction
float opSmoothSubtraction(float d1, float d2, float k) {
  float h = clamp(0.5 - 0.5 * (d2 + d1) / k, 0.0, 1.0);
  return mix(d2, -d1, h) + k * h * (1.0 - h);
}

// Smooth intersection
float opSmoothIntersection(float d1, float d2, float k) {
  float h = clamp(0.5 - 0.5 * (d2 - d1) / k, 0.0, 1.0);
  return mix(d2, d1, h) + k * h * (1.0 - h);
}
```

`k` values: 0.1 = tight blend, 0.5 = soft blend, 1.0+ = very soft.

## SDF Transformations

```glsl
// Translate: shift p before SDF
float d = sdSphere(p - vec3(1.0, 0.0, 0.0), 0.5);

// Rotate (apply inverse rotation to p)
mat2 rot2D(float angle) {
  float s = sin(angle), c = cos(angle);
  return mat2(c, -s, s, c);
}
// Rotate around Y axis:
p.xz = rot2D(uTime) * p.xz;

// Scale (divide p, multiply result)
float d = sdBox(p / scale, vec3(1.0)) * scale;

// Repetition (infinite tiling)
vec3 opRep(vec3 p, vec3 c) {
  return mod(p + 0.5 * c, c) - 0.5 * c;
}

// Limited repetition
vec3 opRepLim(vec3 p, float c, vec3 l) {
  return p - c * clamp(round(p / c), -l, l);
}
```

## Normal Calculation

Approximate surface normal via central differences of the SDF.

```glsl
vec3 calcNormal(vec3 p) {
  vec2 e = vec2(0.001, 0.0);
  return normalize(vec3(
    sceneSDF(p + e.xyy) - sceneSDF(p - e.xyy),
    sceneSDF(p + e.yxy) - sceneSDF(p - e.yxy),
    sceneSDF(p + e.yyx) - sceneSDF(p - e.yyx)
  ));
}
```

## Lighting Models

### Diffuse (Lambert)
```glsl
float diffuse(vec3 normal, vec3 lightDir) {
  return max(dot(normal, lightDir), 0.0);
}
```

### Specular (Blinn-Phong)
```glsl
float specular(vec3 normal, vec3 lightDir, vec3 viewDir, float shininess) {
  vec3 halfDir = normalize(lightDir + viewDir);
  return pow(max(dot(normal, halfDir), 0.0), shininess);
}
```

### Combined Phong shading
```glsl
vec3 shade(vec3 p, vec3 normal, vec3 rd) {
  vec3 lightPos = vec3(2.0, 4.0, -3.0);
  vec3 lightDir = normalize(lightPos - p);
  vec3 viewDir = -rd;

  float amb = 0.1;
  float diff = diffuse(normal, lightDir);
  float spec = specular(normal, lightDir, viewDir, 32.0);

  vec3 color = vec3(0.8, 0.2, 0.3);
  return color * (amb + diff) + vec3(1.0) * spec * 0.5;
}
```

## Shadows & AO

### Soft shadows
```glsl
float softShadow(vec3 ro, vec3 rd, float mint, float maxt, float k) {
  float res = 1.0;
  float t = mint;
  for (int i = 0; i < 64; i++) {
    float d = sceneSDF(ro + rd * t);
    if (d < 0.001) return 0.0;
    res = min(res, k * d / t);
    t += d;
    if (t > maxt) break;
  }
  return res;
}
```

### Ambient Occlusion (AO)
```glsl
float ao(vec3 p, vec3 n) {
  float occ = 0.0;
  float scale = 1.0;
  for (int i = 0; i < 5; i++) {
    float h = 0.01 + 0.12 * float(i);
    float d = sceneSDF(p + n * h);
    occ += (h - d) * scale;
    scale *= 0.95;
  }
  return clamp(1.0 - 3.0 * occ, 0.0, 1.0);
}
```

## Full Scene Example

```glsl
float sceneSDF(vec3 p) {
  float sphere = sdSphere(p - vec3(0.0, 1.0, 0.0), 1.0);
  float ground = sdPlane(p);
  float box = sdBox(p - vec3(2.0, 0.5, 0.0), vec3(0.5));
  return opSmoothUnion(opUnion(sphere, ground), box, 0.3);
}

void main() {
  vec2 uv = (gl_FragCoord.xy - 0.5 * uResolution.xy) / uResolution.y;
  vec3 ro = vec3(0.0, 2.0, -5.0);
  vec3 rd = normalize(vec3(uv, 1.0));

  float t = raymarch(ro, rd);

  vec3 color = vec3(0.1); // background
  if (t < 100.0) {
    vec3 p = ro + rd * t;
    vec3 n = calcNormal(p);
    color = shade(p, n, rd);
    color *= softShadow(p + n * 0.01, normalize(vec3(2.0, 4.0, -3.0)), 0.01, 10.0, 16.0);
    color *= ao(p, n);
  }

  // Gamma correction
  color = pow(color, vec3(1.0 / 2.2));
  gl_FragColor = vec4(color, 1.0);
}
```
