# Post-Processing

## Table of Contents
- [Setup with React Three Fiber](#setup-with-react-three-fiber)
- [Built-in Effects](#built-in-effects)
- [Custom Effects](#custom-effects)
- [Render Targets & Multi-Pass](#render-targets--multi-pass)
- [Screen-Space Techniques](#screen-space-techniques)

## Setup with React Three Fiber

```bash
npm install @react-three/postprocessing postprocessing
```

```tsx
import { EffectComposer, Bloom, Vignette, ChromaticAberration } from '@react-three/postprocessing'

function Scene() {
  return (
    <Canvas>
      <MyScene />
      <EffectComposer>
        <Bloom intensity={1.5} luminanceThreshold={0.9} luminanceSmoothing={0.025} />
        <Vignette offset={0.5} darkness={0.5} />
      </EffectComposer>
    </Canvas>
  )
}
```

The `EffectComposer` from `@react-three/postprocessing` automatically merges compatible effects into a single pass for performance.

## Built-in Effects

| Effect | Key Props | Use Case |
|--------|-----------|----------|
| `Bloom` | `intensity`, `luminanceThreshold`, `luminanceSmoothing` | Glowing lights |
| `DepthOfField` | `focusDistance`, `focalLength`, `bokehScale` | Camera focus blur |
| `Noise` | `opacity`, `premultiply` | Film grain |
| `Vignette` | `offset`, `darkness` | Edge darkening |
| `ChromaticAberration` | `offset` (vec2) | Color fringing |
| `SSAO` | `radius`, `intensity`, `bias` | Ambient occlusion |
| `ToneMapping` | `mode` | HDR → SDR |
| `HueSaturation` | `hue`, `saturation` | Color grading |
| `BrightnessContrast` | `brightness`, `contrast` | Exposure adjustment |
| `DotScreen` | `scale`, `angle` | Halftone dots |
| `Glitch` | `delay`, `duration`, `strength` | Glitch distortion |
| `Pixelation` | `granularity` | Pixel art effect |
| `Scanline` | `density` | CRT scanlines |
| `ColorAverage` | — | Grayscale |

## Custom Effects

Create custom post-processing effects by extending the `Effect` class from `postprocessing`.

```tsx
import { Effect } from 'postprocessing'
import { Uniform } from 'three'

const fragmentShader = /* glsl */`
  uniform float uTime;
  uniform float uStrength;

  void mainImage(const in vec4 inputColor, const in vec2 uv, out vec4 outputColor) {
    // Distort UVs with sine wave
    vec2 distortedUv = uv + vec2(
      sin(uv.y * 20.0 + uTime) * uStrength,
      cos(uv.x * 20.0 + uTime) * uStrength
    );

    // Sample the scene texture at distorted coordinates
    vec4 color = texture2D(inputBuffer, distortedUv);
    outputColor = color;
  }
`

class WaveDistortionEffect extends Effect {
  constructor({ strength = 0.01 } = {}) {
    super('WaveDistortionEffect', fragmentShader, {
      uniforms: new Map([
        ['uTime', new Uniform(0)],
        ['uStrength', new Uniform(strength)],
      ]),
    })
  }

  update(renderer, inputBuffer, deltaTime) {
    this.uniforms.get('uTime').value += deltaTime
  }
}
```

### Using custom effect in R3F

```tsx
import { forwardRef, useMemo } from 'react'

const WaveDistortion = forwardRef(({ strength = 0.01 }, ref) => {
  const effect = useMemo(() => new WaveDistortionEffect({ strength }), [strength])
  return <primitive ref={ref} object={effect} dispose={null} />
})

// In scene:
<EffectComposer>
  <WaveDistortion strength={0.02} />
  <Bloom intensity={1.0} />
</EffectComposer>
```

### Available in custom effect shaders

- `inputBuffer` — scene texture (sampler2D)
- `inputColor` — color at current pixel (passed as parameter)
- `uv` — screen-space coordinates
- Built-in uniforms: `resolution`, `texelSize`, `cameraNear`, `cameraFar`, `aspect`, `time`

### Effect signature

```glsl
// Main function signature (required)
void mainImage(const in vec4 inputColor, const in vec2 uv, out vec4 outputColor) {
  outputColor = inputColor; // passthrough
}

// Optional: for effects that need UV manipulation
void mainUv(inout vec2 uv) {
  uv += 0.01; // shift UVs before mainImage runs
}
```

## Render Targets & Multi-Pass

For complex setups beyond EffectComposer (e.g., capturing scene to texture for portal, reflection, or feedback effects).

### useFBO for render-to-texture

```tsx
import { useFBO } from '@react-three/drei'
import { createPortal, useFrame } from '@react-three/fiber'

function PortalEffect() {
  const fbo = useFBO(512, 512)
  const portalScene = useMemo(() => new THREE.Scene(), [])
  const portalCamera = useRef()

  useFrame((state) => {
    state.gl.setRenderTarget(fbo)
    state.gl.render(portalScene, portalCamera.current)
    state.gl.setRenderTarget(null)
  })

  return (
    <>
      {createPortal(<PortalContent />, portalScene)}
      <mesh>
        <planeGeometry />
        <meshBasicMaterial map={fbo.texture} />
      </mesh>
    </>
  )
}
```

### Ping-pong buffers (feedback effects)

```tsx
const fboA = useFBO(width, height, { type: THREE.FloatType })
const fboB = useFBO(width, height, { type: THREE.FloatType })
const pingPong = useRef(true)

useFrame((state) => {
  const read = pingPong.current ? fboA : fboB
  const write = pingPong.current ? fboB : fboA

  simMaterial.uniforms.uInput.value = read.texture
  state.gl.setRenderTarget(write)
  state.gl.render(simScene, simCamera)
  state.gl.setRenderTarget(null)

  renderMaterial.uniforms.uTexture.value = write.texture
  pingPong.current = !pingPong.current
})
```

## Screen-Space Techniques

### Edge detection (Sobel filter)
```glsl
void mainImage(const in vec4 inputColor, const in vec2 uv, out vec4 outputColor) {
  vec2 texel = texelSize;

  float tl = luminance(texture2D(inputBuffer, uv + vec2(-texel.x,  texel.y)).rgb);
  float t  = luminance(texture2D(inputBuffer, uv + vec2( 0.0,      texel.y)).rgb);
  float tr = luminance(texture2D(inputBuffer, uv + vec2( texel.x,  texel.y)).rgb);
  float l  = luminance(texture2D(inputBuffer, uv + vec2(-texel.x,  0.0)).rgb);
  float r  = luminance(texture2D(inputBuffer, uv + vec2( texel.x,  0.0)).rgb);
  float bl = luminance(texture2D(inputBuffer, uv + vec2(-texel.x, -texel.y)).rgb);
  float b  = luminance(texture2D(inputBuffer, uv + vec2( 0.0,     -texel.y)).rgb);
  float br = luminance(texture2D(inputBuffer, uv + vec2( texel.x, -texel.y)).rgb);

  float gx = -tl - 2.0*l - bl + tr + 2.0*r + br;
  float gy = -tl - 2.0*t - tr + bl + 2.0*b + br;
  float edge = sqrt(gx*gx + gy*gy);

  outputColor = vec4(vec3(edge), 1.0);
}

float luminance(vec3 color) {
  return dot(color, vec3(0.2126, 0.7152, 0.0722));
}
```

### Gaussian blur (separable, two-pass)
```glsl
// Horizontal pass
void mainImage(const in vec4 inputColor, const in vec2 uv, out vec4 outputColor) {
  vec2 texel = texelSize;
  float weights[5] = float[](0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);

  vec3 result = texture2D(inputBuffer, uv).rgb * weights[0];
  for (int i = 1; i < 5; i++) {
    result += texture2D(inputBuffer, uv + vec2(texel.x * float(i), 0.0)).rgb * weights[i];
    result += texture2D(inputBuffer, uv - vec2(texel.x * float(i), 0.0)).rgb * weights[i];
  }
  outputColor = vec4(result, 1.0);
}
```
