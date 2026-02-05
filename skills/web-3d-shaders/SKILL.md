---
name: web-3d-shaders
description: >-
  Shaders and real-time 3D on the Web with GLSL, Three.js, React Three Fiber, and WebGPU.
  Write, review, and refactor shader code for interactive 3D web experiences. Use when:
  (1) Writing GLSL vertex or fragment shaders for Three.js or React Three Fiber,
  (2) Creating shaderMaterial or custom materials with uniforms and varyings,
  (3) Implementing visual effects like noise patterns, raymarching, SDFs, particles, post-processing,
  (4) Working with light effects (refraction, dispersion, Fresnel, caustics, volumetric lighting),
  (5) Creating stylized shaders (painterly/Kuwahara, dithering, retro, Moebius-style),
  (6) Building particle systems with buffer geometries, FBOs, and GPU simulation,
  (7) Migrating shaders from GLSL/WebGL to TSL/WebGPU,
  (8) Optimizing 3D scene performance on the web,
  (9) Setting up React Three Fiber projects with shader effects,
  (10) Any task involving WebGL, WebGPU, GLSL, TSL, Three.js shaders, or React Three Fiber materials.
---

# Shaders & Real-Time 3D on the Web

## Stack Overview

| Layer | WebGL Path | WebGPU Path |
|-------|-----------|-------------|
| Framework | React Three Fiber (`@react-three/fiber`) | React Three Fiber (`@react-three/fiber`) |
| Helpers | Drei (`@react-three/drei`) | Drei (`@react-three/drei`) |
| Post-processing | `@react-three/postprocessing` | `@react-three/postprocessing` |
| Shader language | GLSL (OpenGL Shading Language) | TSL (Three.js Shading Language) |
| Renderer | `THREE.WebGLRenderer` | `THREE.WebGPURenderer` (async init) |

## Project Setup (React Three Fiber)

```bash
npm install three @react-three/fiber @react-three/drei @react-three/postprocessing
```

```tsx
import { Canvas } from '@react-three/fiber'

function App() {
  return (
    <Canvas camera={{ position: [0, 0, 5] }}>
      <ambientLight intensity={0.5} />
      <MyShaderMesh />
    </Canvas>
  )
}
```

## Shader Fundamentals

### Vertex Shader — positions vertices

```glsl
uniform float uTime;
varying vec2 vUv;
varying vec3 vPosition;

void main() {
  vUv = uv;
  vPosition = position;
  vec3 pos = position;
  pos.z += sin(pos.x * 4.0 + uTime) * 0.2;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(pos, 1.0);
}
```

### Fragment Shader — colors pixels

```glsl
uniform float uTime;
varying vec2 vUv;

void main() {
  vec3 color = vec3(vUv.x, vUv.y, sin(uTime) * 0.5 + 0.5);
  gl_FragColor = vec4(color, 1.0);
}
```

### Key GLSL Types

`float`, `vec2`, `vec3`, `vec4` — scalars/vectors | `mat2`, `mat3`, `mat4` — matrices | `sampler2D` — 2D texture | `samplerCube` — cubemap

### Essential GLSL Functions

| Function | Purpose |
|----------|---------|
| `mix(a, b, t)` | Linear interpolation |
| `step(edge, x)` | Hard threshold (0 or 1) |
| `smoothstep(e0, e1, x)` | Smooth threshold |
| `clamp(x, min, max)` | Constrain value |
| `fract(x)` | Fractional part |
| `mod(x, y)` | Modulo |
| `length(v)` | Vector magnitude |
| `normalize(v)` | Unit vector |
| `dot(a, b)` | Dot product |
| `cross(a, b)` | Cross product |
| `reflect(I, N)` | Reflection vector |
| `refract(I, N, eta)` | Refraction vector |
| `texture2D(s, uv)` | Sample texture |

### Uniforms, Varyings, Attributes

- **Uniforms**: JS → shader, constant per draw call (time, resolution, mouse, textures)
- **Varyings**: Vertex → fragment, interpolated per pixel (UVs, normals, positions)
- **Attributes**: Per-vertex data (position, uv, normal, custom per-vertex data)

## React Three Fiber Integration

### shaderMaterial with Drei

```tsx
import { shaderMaterial } from '@react-three/drei'
import { extend, useFrame } from '@react-three/fiber'
import { useRef, useMemo } from 'react'
import * as THREE from 'three'

const MyMaterial = shaderMaterial(
  { uTime: 0, uColor: new THREE.Color(0.2, 0.0, 0.1) },
  // vertex
  /* glsl */`
    varying vec2 vUv;
    void main() {
      vUv = uv;
      gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
    }
  `,
  // fragment
  /* glsl */`
    uniform float uTime;
    uniform vec3 uColor;
    varying vec2 vUv;
    void main() {
      gl_FragColor = vec4(uColor + sin(uTime) * 0.3, 1.0);
    }
  `
)

extend({ MyMaterial })

function MyMesh() {
  const ref = useRef()
  useFrame((state) => {
    ref.current.uTime = state.clock.elapsedTime
  })
  return (
    <mesh>
      <planeGeometry args={[2, 2, 32, 32]} />
      <myMaterial ref={ref} />
    </mesh>
  )
}
```

### Raw THREE.ShaderMaterial

```tsx
function MyMesh() {
  const uniforms = useMemo(() => ({
    uTime: { value: 0 },
    uTexture: { value: new THREE.TextureLoader().load('/texture.png') },
  }), [])

  useFrame((state) => {
    uniforms.uTime.value = state.clock.elapsedTime
  })

  return (
    <mesh>
      <boxGeometry />
      <shaderMaterial
        vertexShader={vertexShader}
        fragmentShader={fragmentShader}
        uniforms={uniforms}
        transparent
      />
    </mesh>
  )
}
```

**Critical**: Memoize `uniforms` with `useMemo` to prevent re-creation on re-render. Update values via `uniforms.uTime.value` inside `useFrame`.

## Technique References

Consult these files for detailed patterns on specific techniques:

- **Noise & Procedural Patterns**: See [references/noise-patterns.md](references/noise-patterns.md) — Perlin, Simplex, FBM, domain warping, Voronoi
- **Raymarching & SDFs**: See [references/raymarching-sdf.md](references/raymarching-sdf.md) — SDF primitives, boolean ops, smooth blending, lighting
- **Particles & GPU Simulation**: See [references/particles-gpu.md](references/particles-gpu.md) — Buffer geometries, FBOs, GPGPU, instancing
- **Post-Processing**: See [references/post-processing.md](references/post-processing.md) — EffectComposer, custom effects, render targets
- **Light Effects**: See [references/light-effects.md](references/light-effects.md) — Refraction, dispersion, Fresnel, caustics, volumetric
- **Stylized Shaders**: See [references/stylized-shaders.md](references/stylized-shaders.md) — Kuwahara, dithering, Moebius-style, retro
- **TSL & WebGPU**: See [references/tsl-webgpu.md](references/tsl-webgpu.md) — TSL node system, compute shaders, GLSL migration

## Performance Best Practices

1. **Minimize draw calls** — use `InstancedMesh` for repeated geometry
2. **Offload to GPU** — move computation to shaders (particles, physics, animations)
3. **Use FBOs wisely** — render targets enable ping-pong/GPGPU but cost a full render pass
4. **Reduce overdraw** — sort transparent objects, use `depthWrite`/`depthTest`
5. **Keep shaders lean** — avoid deep loops; prefer analytical over iterative solutions
6. **Use `lowp`/`mediump`** precision where possible in fragment shaders
7. **Dispose resources** — `.dispose()` on geometries, materials, textures, render targets
8. **Limit post-processing passes** — merge effects when possible
9. **Animate with delta time** — use `useFrame` delta for consistent frame rates

## Debugging Tips

- Visualize data as color: `gl_FragColor = vec4(vUv, 0.0, 1.0);`
- Visualize normals: `gl_FragColor = vec4(normal * 0.5 + 0.5, 1.0);`
- Check uniform updates by outputting as color channel
- Clamp values to catch NaN/Inf from division by zero
- Use Spector.js or WebGL Inspector for draw call debugging

## Key Resources

- [The Book of Shaders](https://thebookofshaders.com/) — Interactive GLSL fundamentals
- [Shadertoy](https://www.shadertoy.com/) — Community shader experiments
- [Inigo Quilez](https://iquilezles.org/articles/) — SDF, raymarching, math
- [Three.js Journey](https://threejs-journey.com/) — Comprehensive Three.js course
- [Maxime Heckel's blog](https://blog.maximeheckel.com/) — In-depth R3F shader tutorials
- [Poimandres Discord](https://discord.gg/poimandres) — R3F community
- [Lygia shader library](https://lygia.xyz/) — Reusable GLSL/HLSL functions
