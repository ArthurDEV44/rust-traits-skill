# TSL & WebGPU

## Table of Contents
- [Overview](#overview)
- [WebGPU Renderer Setup](#webgpu-renderer-setup)
- [TSL Basics](#tsl-basics)
- [TSL Node Types](#tsl-node-types)
- [Custom Materials with TSL](#custom-materials-with-tsl)
- [Compute Shaders](#compute-shaders)
- [GLSL to TSL Migration](#glsl-to-tsl-migration)

## Overview

**TSL (Three.js Shading Language)** is Three.js's node-based material system (since r166). Write shaders in JavaScript that compile to both WGSL (WebGPU) and GLSL (WebGL). TSL replaces raw GLSL strings with composable JS functions.

**WebGPU** is the modern graphics API successor to WebGL, with compute shader support and better performance. Supported in Chrome, Edge, Firefox, and Safari (since Safari 26, Sept 2025).

Key advantages of TSL:
- Write once, run on both WebGL and WebGPU
- JavaScript stack traces instead of cryptic GLSL errors
- Composable, reusable shader functions
- Access to compute shaders (WebGPU only)

## WebGPU Renderer Setup

```tsx
import WebGPURenderer from 'three/webgpu'

// Async initialization required
const renderer = new WebGPURenderer()
await renderer.init()
renderer.setSize(window.innerWidth, window.innerHeight)
document.body.appendChild(renderer.domElement)
```

### R3F with WebGPU
```tsx
import { Canvas } from '@react-three/fiber'

// R3F handles async init when you specify the frameloop
<Canvas
  gl={(canvas) => {
    const renderer = new WebGPURenderer({ canvas })
    return renderer
  }}
>
  <Scene />
</Canvas>
```

## TSL Basics

Import node functions from `three/tsl`:

```js
import {
  uniform, attribute, varying, texture,
  float, vec2, vec3, vec4, color, int,
  Fn, If,
  positionLocal, positionWorld, normalLocal, normalWorld,
  modelWorldMatrix, cameraPosition,
  uv, time,
  mix, step, smoothstep, clamp, fract, sin, cos, abs, length, normalize, dot, cross, reflect,
  mul, add, sub, div, mod,
  max, min, pow,
} from 'three/tsl'
```

### Basic TSL Material

```js
import { MeshStandardNodeMaterial } from 'three/webgpu'
import { uv, sin, time, vec3, color, mix } from 'three/tsl'

const material = new MeshStandardNodeMaterial()

// Override color output
material.colorNode = mix(
  color(0xff0000),
  color(0x0000ff),
  sin(time).mul(0.5).add(0.5)
)

// Override position
material.positionNode = positionLocal.add(
  vec3(0, sin(positionLocal.x.mul(4).add(time)).mul(0.2), 0)
)
```

## TSL Node Types

### Uniforms
```js
const uIntensity = uniform(float(1.0))
const uColor = uniform(color('#ff0000'))

// Update from JS
uIntensity.value = 2.0
```

### Varyings
```js
import { varying, vec2, uv } from 'three/tsl'

const vUv = varying(vec2(), 'vUv')
// In vertex: vUv.assign(uv())
// In fragment: use vUv
```

### Built-in nodes
| Node | GLSL Equivalent |
|------|----------------|
| `positionLocal` | `position` (model space) |
| `positionWorld` | world space position |
| `normalLocal` | `normal` (model space) |
| `normalWorld` | world space normal |
| `uv()` | `uv` attribute |
| `time` | elapsed time uniform |
| `cameraPosition` | camera world position |

### Math operations
```js
// TSL uses method chaining
const result = a.add(b).mul(c)           // (a + b) * c
const clamped = val.clamp(0, 1)          // clamp(val, 0, 1)
const mixed = mix(colorA, colorB, t)     // mix(a, b, t)
const stepped = smoothstep(0.3, 0.7, x)  // smoothstep(0.3, 0.7, x)
```

## Custom Materials with TSL

### Fn — define reusable shader functions

```js
import { Fn, float, vec3, vec4 } from 'three/tsl'

const myNoise = Fn(([p_immutable]) => {
  const p = vec2(p_immutable)
  return fract(sin(dot(p, vec2(127.1, 311.7))).mul(43758.5453123))
})

// Use in material
material.colorNode = vec3(myNoise(uv().mul(10)))
```

### Full custom node material
```js
import { MeshStandardNodeMaterial } from 'three/webgpu'

const material = new MeshStandardNodeMaterial()

// Vertex displacement
const displacement = sin(positionLocal.x.mul(4).add(time)).mul(0.2)
material.positionNode = positionLocal.add(normalLocal.mul(displacement))

// Fragment coloring
const fresnel = float(1).sub(dot(normalWorld, normalize(cameraPosition.sub(positionWorld))).max(0)).pow(3)
material.colorNode = mix(color('#1a1a2e'), color('#e94560'), fresnel)
material.emissiveNode = color('#e94560').mul(fresnel).mul(0.5)
```

## Compute Shaders

WebGPU-only. Run arbitrary GPU computation, ideal for particle simulation.

```js
import { compute, storage, float, vec4, instanceIndex } from 'three/tsl'
import { StorageBufferAttribute } from 'three/webgpu'

// Create storage buffer
const count = 100000
const positionBuffer = new StorageBufferAttribute(new Float32Array(count * 4), 4)

// Define compute function
const computeParticles = Fn(() => {
  const pos = storage(positionBuffer, 'vec4', count).element(instanceIndex)
  const vel = vec4(sin(float(instanceIndex).mul(0.01).add(time)), 0, 0, 0)
  pos.addAssign(vel.mul(0.01))
})

// Create compute node
const computeNode = computeParticles().compute(count)

// In render loop
renderer.compute(computeNode)
```

## GLSL to TSL Migration

| GLSL | TSL |
|------|-----|
| `float x = 1.0;` | `const x = float(1.0)` |
| `vec3 c = vec3(1.0, 0.0, 0.0);` | `const c = vec3(1, 0, 0)` |
| `x + y` | `x.add(y)` |
| `x * y` | `x.mul(y)` |
| `sin(x)` | `sin(x)` |
| `mix(a, b, t)` | `mix(a, b, t)` |
| `smoothstep(e0, e1, x)` | `smoothstep(e0, e1, x)` |
| `dot(a, b)` | `dot(a, b)` |
| `normalize(v)` | `normalize(v)` |
| `length(v)` | `length(v)` |
| `texture2D(tex, uv)` | `texture(tex, uv)` |
| `if (x > 0.0) { ... }` | `If(x.greaterThan(0), () => { ... })` |
| `uniform float uTime;` | `const uTime = uniform(float(0))` |
| `varying vec2 vUv;` | `const vUv = varying(vec2())` |
| `gl_Position = ...` | `material.positionNode = ...` |
| `gl_FragColor = ...` | `material.colorNode = ...` |
| `attribute vec3 position;` | `positionLocal` |
| `uv` | `uv()` |

### Material node outputs

| Node Property | GLSL Equivalent |
|--------------|----------------|
| `material.positionNode` | Vertex position output |
| `material.colorNode` | Diffuse/albedo color |
| `material.normalNode` | Surface normal |
| `material.emissiveNode` | Emissive color |
| `material.roughnessNode` | PBR roughness |
| `material.metalnessNode` | PBR metalness |
| `material.opacityNode` | Alpha/transparency |
| `material.fragmentNode` | Full fragment output (bypasses PBR) |

### Key differences
- TSL is **JavaScript** — use JS variables, loops, conditionals, imports
- Operations are **method-chained** — `a.add(b).mul(c)` not `(a + b) * c`
- Use `Fn` to define reusable shader functions
- `If` for conditionals (not JS `if` — shader compilation is different)
- Debugging gives **JS stack traces** not GLSL compilation errors
- TSL is still evolving; some advanced GLSL patterns may not have direct equivalents
