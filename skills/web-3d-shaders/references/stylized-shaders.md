# Stylized Shaders

## Table of Contents
- [Kuwahara Filter (Painterly)](#kuwahara-filter-painterly)
- [Dithering & Retro Shading](#dithering--retro-shading)
- [Moebius-Style Post-Processing](#moebius-style-post-processing)
- [Toon / Cel Shading](#toon--cel-shading)
- [Halftone](#halftone)
- [Pixel Art / PS1 Style](#pixel-art--ps1-style)
- [Watercolor Effect](#watercolor-effect)

## Kuwahara Filter (Painterly)

Smooths images while preserving edges, creating an oil-painting look. Divides a kernel into four quadrants, selects the one with lowest variance.

```glsl
// Basic Kuwahara filter
vec4 kuwahara(sampler2D tex, vec2 uv, vec2 texelSize, int radius) {
  vec3 mean[4];
  vec3 sigma[4];

  for (int i = 0; i < 4; i++) {
    mean[i] = vec3(0.0);
    sigma[i] = vec3(0.0);
  }

  int r = radius;
  float count = float((r + 1) * (r + 1));

  // Four quadrants: top-left, top-right, bottom-left, bottom-right
  for (int j = -r; j <= 0; j++) {
    for (int i = -r; i <= 0; i++) {
      vec3 c = texture2D(tex, uv + vec2(float(i), float(j)) * texelSize).rgb;
      mean[0] += c;
      sigma[0] += c * c;
    }
  }
  // ... repeat for other 3 quadrants with different ranges

  // Normalize and compute variance for each quadrant
  for (int i = 0; i < 4; i++) {
    mean[i] /= count;
    sigma[i] = abs(sigma[i] / count - mean[i] * mean[i]);
  }

  // Select quadrant with minimum variance
  float minSigma = 1e10;
  vec3 result = mean[0];
  for (int i = 0; i < 4; i++) {
    float s = sigma[i].r + sigma[i].g + sigma[i].b;
    if (s < minSigma) {
      minSigma = s;
      result = mean[i];
    }
  }

  return vec4(result, 1.0);
}
```

Larger radius = more painted feel. Cost grows quadratically — use separable or anisotropic variants for performance.

### Anisotropic Kuwahara
Follow edge direction from a structure tensor for brush-stroke-aligned filtering. More expensive but produces directional brush strokes.

## Dithering & Retro Shading

Simulate limited color palettes using dithering patterns.

### Ordered dithering (Bayer matrix)
```glsl
// 4x4 Bayer matrix
float bayer4x4(vec2 pos) {
  int x = int(mod(pos.x, 4.0));
  int y = int(mod(pos.y, 4.0));
  // 4x4 Bayer threshold matrix (normalized to 0-1)
  float matrix[16] = float[16](
     0.0/16.0,  8.0/16.0,  2.0/16.0, 10.0/16.0,
    12.0/16.0,  4.0/16.0, 14.0/16.0,  6.0/16.0,
     3.0/16.0, 11.0/16.0,  1.0/16.0,  9.0/16.0,
    15.0/16.0,  7.0/16.0, 13.0/16.0,  5.0/16.0
  );
  return matrix[y * 4 + x];
}

// Apply dithering to reduce color depth
void mainImage(const in vec4 inputColor, const in vec2 uv, out vec4 outputColor) {
  vec3 color = inputColor.rgb;
  float levels = 4.0; // number of color levels per channel
  float threshold = bayer4x4(gl_FragCoord.xy);

  // Quantize with dithering
  color = floor(color * levels + threshold) / levels;
  outputColor = vec4(color, 1.0);
}
```

### Blue noise dithering
Use a blue noise texture instead of Bayer for more organic, less patterned results:
```glsl
float noise = texture2D(uBlueNoise, gl_FragCoord.xy / 256.0).r;
color = floor(color * levels + noise) / levels;
```

## Moebius-Style Post-Processing

Combines edge detection (Sobel), custom lighting, and noise displacement for a hand-drawn comic look.

```glsl
void mainImage(const in vec4 inputColor, const in vec2 uv, out vec4 outputColor) {
  // 1. Edge detection
  float edge = sobelEdge(uv); // see post-processing.md

  // 2. Noise displacement for hand-drawn feel
  float noise = texture2D(uNoiseTexture, uv * 3.0).r;
  vec2 displacedUv = uv + (noise - 0.5) * 0.003;

  // 3. Quantize brightness into bands
  vec3 color = texture2D(inputBuffer, displacedUv).rgb;
  float lum = dot(color, vec3(0.299, 0.587, 0.114));
  float bands = floor(lum * 4.0) / 4.0;

  // 4. Apply hatching based on brightness
  float hatch = 1.0;
  if (bands < 0.5) {
    hatch = step(0.5, fract(uv.x * 200.0 + uv.y * 50.0));
  }
  if (bands < 0.25) {
    hatch *= step(0.5, fract(uv.x * 50.0 - uv.y * 200.0));
  }

  // 5. Combine
  vec3 result = vec3(bands * hatch);
  result *= (1.0 - edge * 0.8); // darken edges
  outputColor = vec4(result, 1.0);
}
```

## Toon / Cel Shading

Quantize lighting into discrete bands with hard outlines.

```glsl
// In fragment shader (on 3D objects)
float diff = max(dot(normal, lightDir), 0.0);

// Quantize into bands
float toon;
if (diff > 0.9) toon = 1.0;
else if (diff > 0.5) toon = 0.7;
else if (diff > 0.2) toon = 0.4;
else toon = 0.2;

vec3 color = baseColor * toon;

// Add rim light for silhouette
float rim = 1.0 - max(dot(viewDir, normal), 0.0);
rim = smoothstep(0.6, 1.0, rim);
color += rimColor * rim;
```

Combine with Sobel edge detection in post-processing for outlines.

## Halftone

Dot patterns varying in size based on brightness.

```glsl
void mainImage(const in vec4 inputColor, const in vec2 uv, out vec4 outputColor) {
  float dotSize = 8.0; // pixels per dot
  vec2 center = floor(gl_FragCoord.xy / dotSize) * dotSize + dotSize * 0.5;
  vec2 sampleUv = center / uResolution;

  float lum = dot(texture2D(inputBuffer, sampleUv).rgb, vec3(0.299, 0.587, 0.114));
  float radius = dotSize * 0.5 * (1.0 - lum);
  float dist = length(gl_FragCoord.xy - center);

  float dot = 1.0 - step(radius, dist);
  outputColor = vec4(vec3(dot), 1.0);
}
```

### CMYK halftone
Apply separate dot patterns at different angles for C, M, Y, K channels (15, 75, 0, 45 degrees).

## Pixel Art / PS1 Style

### Resolution reduction
```glsl
void mainImage(const in vec4 inputColor, const in vec2 uv, out vec4 outputColor) {
  float pixelSize = 4.0;
  vec2 pixelatedUv = floor(uv * uResolution / pixelSize) * pixelSize / uResolution;
  outputColor = texture2D(inputBuffer, pixelatedUv);
}
```

### PS1 vertex snapping (in vertex shader)
```glsl
void main() {
  vec4 pos = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
  // Snap vertices to grid (simulates low-precision fixed-point math)
  float gridSize = 160.0;
  pos.xyz = floor(pos.xyz * gridSize) / gridSize;
  gl_Position = pos;
}
```

### Affine texture mapping (PS1)
```glsl
// Vertex shader: pass uninterpolated UVs
varying vec2 vUv;
varying float vW;
void main() {
  vUv = uv;
  vec4 pos = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
  vW = pos.w;
  vUv *= vW; // cancel perspective correction
  gl_Position = pos;
}

// Fragment shader
varying vec2 vUv;
varying float vW;
void main() {
  vec2 uv = vUv / vW; // affine (no perspective correction)
  gl_FragColor = texture2D(uTexture, uv);
}
```

## Watercolor Effect

Combine multiple techniques for a watercolor look.

```glsl
void mainImage(const in vec4 inputColor, const in vec2 uv, out vec4 outputColor) {
  // 1. Wobble edges with noise
  float noise = texture2D(uNoiseTex, uv * 2.0).r;
  vec2 wobbleUv = uv + (noise - 0.5) * 0.01;

  // 2. Kuwahara-filtered base
  vec3 painted = kuwahara(inputBuffer, wobbleUv, texelSize, 5).rgb;

  // 3. Desaturate slightly
  float lum = dot(painted, vec3(0.3, 0.59, 0.11));
  painted = mix(vec3(lum), painted, 0.8);

  // 4. Paper texture overlay
  float paper = texture2D(uPaperTexture, uv * 3.0).r;
  painted *= mix(0.9, 1.1, paper);

  // 5. Edge darkening
  float edge = sobelEdge(uv);
  painted *= (1.0 - edge * 0.3);

  outputColor = vec4(painted, 1.0);
}
```
