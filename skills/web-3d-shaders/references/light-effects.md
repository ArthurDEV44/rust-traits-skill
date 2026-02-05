# Light Effects

## Table of Contents
- [Fresnel Effect](#fresnel-effect)
- [Refraction](#refraction)
- [Chromatic Dispersion](#chromatic-dispersion)
- [Chromatic Aberration](#chromatic-aberration)
- [Specular Highlights](#specular-highlights)
- [Caustics](#caustics)
- [Volumetric Lighting](#volumetric-lighting)
- [Iridescence](#iridescence)

## Fresnel Effect

Surfaces reflect more light at grazing angles. Essential for glass, water, and metallic surfaces.

```glsl
float fresnel(vec3 viewDir, vec3 normal, float power) {
  return pow(1.0 - max(dot(viewDir, normal), 0.0), power);
}

// Usage in fragment shader
vec3 viewDir = normalize(cameraPosition - vWorldPosition);
float f = fresnel(viewDir, vNormal, 2.0);
vec3 color = mix(baseColor, reflectionColor, f);
```

Higher `power` = narrower edge effect. Typical range: 1.0 to 5.0.

### Schlick's Approximation (physically-based)
```glsl
float fresnelSchlick(float cosTheta, float F0) {
  return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}
// F0 = reflectance at normal incidence (0.04 for dielectrics, 0.5-1.0 for metals)
```

## Refraction

Bend light rays through transparent surfaces using `refract()` and FBO textures.

```glsl
uniform sampler2D uSceneTexture; // scene rendered to FBO
uniform float uIOR; // index of refraction (glass ~1.5, water ~1.33)

varying vec3 vNormal;
varying vec3 vViewDir;

void main() {
  vec3 normal = normalize(vNormal);
  vec3 viewDir = normalize(vViewDir);

  // Refract the view direction
  vec3 refracted = refract(viewDir, normal, 1.0 / uIOR);

  // Project refracted ray to screen space for texture lookup
  vec2 uv = gl_FragCoord.xy / uResolution;
  vec2 offset = refracted.xy * 0.1; // strength of distortion
  vec4 color = texture2D(uSceneTexture, uv + offset);

  gl_FragColor = color;
}
```

**IOR values**: Air 1.0, Water 1.33, Glass 1.5, Diamond 2.42, Crystal 2.0.

## Chromatic Dispersion

Different wavelengths refract at different angles, splitting white light into colors.

```glsl
uniform float uIOR;
uniform float uDispersion; // spread between channels

void main() {
  vec3 normal = normalize(vNormal);
  vec3 viewDir = normalize(vViewDir);
  vec2 uv = gl_FragCoord.xy / uResolution;

  // Different IOR per color channel
  float iorR = uIOR - uDispersion;
  float iorG = uIOR;
  float iorB = uIOR + uDispersion;

  vec3 refractR = refract(viewDir, normal, 1.0 / iorR);
  vec3 refractG = refract(viewDir, normal, 1.0 / iorG);
  vec3 refractB = refract(viewDir, normal, 1.0 / iorB);

  float r = texture2D(uSceneTexture, uv + refractR.xy * 0.1).r;
  float g = texture2D(uSceneTexture, uv + refractG.xy * 0.1).g;
  float b = texture2D(uSceneTexture, uv + refractB.xy * 0.1).b;

  gl_FragColor = vec4(r, g, b, 1.0);
}
```

## Chromatic Aberration

Screen-space color channel offset (lens artifact), simpler than physical dispersion.

```glsl
// Post-processing effect
void mainImage(const in vec4 inputColor, const in vec2 uv, out vec4 outputColor) {
  vec2 dir = uv - 0.5;
  float dist = length(dir);

  vec2 offset = dir * dist * uStrength;

  float r = texture2D(inputBuffer, uv + offset).r;
  float g = texture2D(inputBuffer, uv).g;
  float b = texture2D(inputBuffer, uv - offset).b;

  outputColor = vec4(r, g, b, 1.0);
}
```

## Specular Highlights

### Phong
```glsl
vec3 reflectDir = reflect(-lightDir, normal);
float spec = pow(max(dot(viewDir, reflectDir), 0.0), shininess);
```

### Blinn-Phong (preferred — cheaper, more physically plausible)
```glsl
vec3 halfDir = normalize(lightDir + viewDir);
float spec = pow(max(dot(normal, halfDir), 0.0), shininess);
```

### GGX / Cook-Torrance (PBR)
```glsl
float D_GGX(float NdotH, float roughness) {
  float a = roughness * roughness;
  float a2 = a * a;
  float denom = NdotH * NdotH * (a2 - 1.0) + 1.0;
  return a2 / (3.14159 * denom * denom);
}
```

## Caustics

Light patterns focused through curved transparent surfaces. Approaches:

### Texture-based caustics
```glsl
uniform sampler2D uCausticsTexture;
uniform float uTime;

// Project caustics onto surfaces
vec2 causticsUv = vWorldPosition.xz * 0.5 + uTime * 0.05;
float caustics = texture2D(uCausticsTexture, causticsUv).r;
caustics += texture2D(uCausticsTexture, causticsUv * 1.5 + 0.3).r;
color += caustics * 0.3 * lightColor;
```

### Voronoi-based caustics
```glsl
float caustics(vec2 uv) {
  float v1 = voronoi(uv * 5.0 + uTime * 0.2);
  float v2 = voronoi(uv * 5.0 + uTime * 0.2 + 0.5);
  // Sharp bright lines at cell edges
  return pow(min(v1, v2), 3.0) * 5.0;
}
```

## Volumetric Lighting

God rays / light shafts via screen-space raymarching in post-processing.

### Radial blur approach
```glsl
uniform vec2 uLightScreenPos; // light position in screen space [0-1]
uniform float uDecay;
uniform float uDensity;
uniform float uWeight;
uniform int uSamples;

void mainImage(const in vec4 inputColor, const in vec2 uv, out vec4 outputColor) {
  vec2 deltaUv = (uv - uLightScreenPos) * (1.0 / float(uSamples)) * uDensity;
  vec2 sampleUv = uv;
  float illumination = 1.0;
  vec3 color = vec3(0.0);

  for (int i = 0; i < 100; i++) {
    if (i >= uSamples) break;
    sampleUv -= deltaUv;
    vec3 sample = texture2D(inputBuffer, sampleUv).rgb;
    sample *= illumination * uWeight;
    color += sample;
    illumination *= uDecay;
  }

  outputColor = vec4(inputColor.rgb + color, 1.0);
}
```

### Raymarched volumetric fog
```glsl
// March through scene from camera, accumulate density at each step
vec3 volumetric(vec3 ro, vec3 rd, float maxDist) {
  vec3 light = vec3(0.0);
  float stepSize = maxDist / float(STEPS);
  float t = 0.0;

  for (int i = 0; i < STEPS; i++) {
    vec3 p = ro + rd * t;
    float density = fbm(p * 0.5 + uTime * 0.1) * 0.5;

    // Light contribution at this point
    float lightDist = length(uLightPos - p);
    float attenuation = 1.0 / (1.0 + lightDist * lightDist * 0.1);
    light += density * attenuation * uLightColor * stepSize;

    t += stepSize;
  }
  return light;
}
```

## Iridescence

Thin-film interference creating angle-dependent rainbow colors.

```glsl
vec3 iridescence(float cosTheta, float thickness) {
  // Thin film interference
  float delta = 2.0 * thickness * cosTheta;
  vec3 color;
  color.r = cos(delta * 2.0) * 0.5 + 0.5;
  color.g = cos(delta * 2.5) * 0.5 + 0.5;
  color.b = cos(delta * 3.0) * 0.5 + 0.5;
  return color;
}

// Usage
float cosTheta = dot(viewDir, normal);
vec3 iriColor = iridescence(cosTheta, uThickness);
vec3 finalColor = mix(baseColor, iriColor, uIridescenceStrength);
```
