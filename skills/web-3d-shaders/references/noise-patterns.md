# Noise & Procedural Patterns

## Table of Contents
- [Noise Types](#noise-types)
- [Perlin Noise](#perlin-noise)
- [Simplex Noise](#simplex-noise)
- [Value Noise](#value-noise)
- [Voronoi / Worley Noise](#voronoi--worley-noise)
- [Fractal Brownian Motion (FBM)](#fractal-brownian-motion-fbm)
- [Domain Warping](#domain-warping)
- [Common Patterns](#common-patterns)

## Noise Types

| Type | Cost | Quality | Use Case |
|------|------|---------|----------|
| Value noise | Low | Blocky artifacts | Quick prototyping |
| Perlin noise | Medium | Smooth, directional bias | Terrain, displacement |
| Simplex noise | Medium | Less artifacts than Perlin | General purpose |
| Voronoi/Worley | Higher | Cell-like patterns | Organic textures, caustics |

## Perlin Noise

Classic gradient noise. Use `cnoise()` from webgl-noise or implement manually.

```glsl
// 2D Perlin noise (from stegu/webgl-noise)
// Include via glslify: #pragma glslify: cnoise = require(glsl-noise/classic/2d)

// Usage
float n = cnoise(vUv * 10.0 + uTime * 0.5);
// n ranges roughly -1.0 to 1.0, remap as needed:
float remapped = n * 0.5 + 0.5; // 0.0 to 1.0
```

## Simplex Noise

Less directional artifacts than Perlin, slightly cheaper in higher dimensions.

```glsl
// 2D Simplex noise
// #pragma glslify: snoise = require(glsl-noise/simplex/2d)

float n = snoise(vUv * 8.0 + uTime);

// 3D simplex for animated surfaces
float n3d = snoise(vec3(vUv * 5.0, uTime * 0.3));
```

## Value Noise

Cheapest noise. Interpolates random values at grid points.

```glsl
float hash(vec2 p) {
  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123);
}

float valueNoise(vec2 p) {
  vec2 i = floor(p);
  vec2 f = fract(p);
  f = f * f * (3.0 - 2.0 * f); // smoothstep interpolation

  float a = hash(i);
  float b = hash(i + vec2(1.0, 0.0));
  float c = hash(i + vec2(0.0, 1.0));
  float d = hash(i + vec2(1.0, 1.0));

  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}
```

## Voronoi / Worley Noise

Cell-like patterns. Each point finds its nearest random feature point.

```glsl
float voronoi(vec2 p) {
  vec2 i = floor(p);
  vec2 f = fract(p);
  float minDist = 1.0;

  for (int y = -1; y <= 1; y++) {
    for (int x = -1; x <= 1; x++) {
      vec2 neighbor = vec2(float(x), float(y));
      vec2 point = hash2(i + neighbor); // random vec2 per cell
      vec2 diff = neighbor + point - f;
      minDist = min(minDist, length(diff));
    }
  }
  return minDist;
}
```

**Variations**: Use `minDist` for cell edges, second-closest for cell boundaries, difference for crackle patterns.

## Fractal Brownian Motion (FBM)

Layer multiple octaves of noise with increasing frequency and decreasing amplitude for natural complexity.

```glsl
float fbm(vec2 p) {
  float value = 0.0;
  float amplitude = 0.5;
  float frequency = 1.0;
  // Typical: 4-8 octaves
  for (int i = 0; i < 6; i++) {
    value += amplitude * snoise(p * frequency);
    frequency *= 2.0;    // lacunarity (frequency multiplier)
    amplitude *= 0.5;    // gain (amplitude multiplier)
  }
  return value;
}
```

**Parameters**:
- **Octaves** (loop count): More = finer detail, higher cost
- **Lacunarity** (frequency multiplier): Typically 2.0
- **Gain** (amplitude multiplier): Typically 0.5 (persistence)

## Domain Warping

Feed noise output back as input coordinates for organic, flowing distortions.

```glsl
float warpedNoise(vec2 p) {
  // First pass: distort coordinates
  vec2 q = vec2(
    fbm(p + vec2(0.0, 0.0)),
    fbm(p + vec2(5.2, 1.3))
  );

  // Second pass: use distorted coords
  vec2 r = vec2(
    fbm(p + 4.0 * q + vec2(1.7, 9.2) + 0.15 * uTime),
    fbm(p + 4.0 * q + vec2(8.3, 2.8) + 0.126 * uTime)
  );

  return fbm(p + 4.0 * r);
}
```

Multi-layer domain warping creates painterly, fluid effects. Animate with `uTime` for flowing motion.

## Common Patterns

### Turbulence (absolute value FBM)
```glsl
float turbulence(vec2 p) {
  float value = 0.0;
  float amplitude = 0.5;
  float frequency = 1.0;
  for (int i = 0; i < 6; i++) {
    value += amplitude * abs(snoise(p * frequency));
    frequency *= 2.0;
    amplitude *= 0.5;
  }
  return value;
}
```

### Ridged noise
```glsl
float ridged(vec2 p) {
  float value = 0.0;
  float amplitude = 0.5;
  float frequency = 1.0;
  for (int i = 0; i < 6; i++) {
    float n = 1.0 - abs(snoise(p * frequency));
    n = n * n; // sharpen ridges
    value += amplitude * n;
    frequency *= 2.0;
    amplitude *= 0.5;
  }
  return value;
}
```

### Noise-based displacement (vertex shader)
```glsl
void main() {
  vUv = uv;
  vec3 pos = position;
  float displacement = fbm(pos.xz * 2.0 + uTime * 0.2) * 0.5;
  pos.y += displacement;
  // Recompute normals for lighting after displacement
  gl_Position = projectionMatrix * modelViewMatrix * vec4(pos, 1.0);
}
```

### Noise for color mixing
```glsl
void main() {
  float n = fbm(vUv * 5.0 + uTime * 0.1);
  vec3 color1 = vec3(0.1, 0.3, 0.8);
  vec3 color2 = vec3(0.9, 0.2, 0.3);
  vec3 color = mix(color1, color2, smoothstep(-0.2, 0.2, n));
  gl_FragColor = vec4(color, 1.0);
}
```
