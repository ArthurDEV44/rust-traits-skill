---
name: tsl-webgpu
description: "TSL (Three.js Shading Language) and WebGPU shader development with Three.js. Write, review, and refactor GPU shaders using TSL's node system, compute shaders, and WebGPU renderer. Use when: (1) Setting up WebGPURenderer in Three.js or React Three Fiber, (2) Writing TSL shaders with Fn, uniform, varying, texture, (3) Creating or modifying node materials (MeshStandardNodeMaterial, SpriteNodeMaterial, etc.), (4) Using positionNode, colorNode, normalNode or other material nodes, (5) Writing compute shaders with instancedArray, storage, compute(), computeAsync(), (6) Building particle systems with sprite and SpriteNodeMaterial, (7) Post-processing with TSL (pass, bloom, fxaa, PostProcessing class), (8) Porting GLSL shaders to TSL or WGSL, (9) Using wgslFn, glslFn, or code() for native shader code, (10) Working with StorageTexture, textureStore, or GPU buffers, (11) Implementing instanced meshes with compute pipeline, (12) Any Three.js WebGPU or TSL related development."
---

# TSL & WebGPU Development Guide

## Key Concepts

**TSL** = Three.js Shading Language. A functional, JavaScript-based shading language that compiles to both WGSL (WebGPU) and GLSL (WebGL). Write once, run on both backends.

**WebGPURenderer** supports both WebGPU and WebGL fallback. Use `forceWebGL: true` to test WebGL compatibility.

**Node System** = Material properties (color, position, normals) are overridden via node props (`colorNode`, `positionNode`, `normalNode`), replacing the old `onBeforeCompile` approach.

**Compute Shaders** = WebGPU-only. Run arbitrary GPU computation via the compute pipeline. Use `instancedArray` for buffers, `.compute(COUNT)` to dispatch, `renderer.compute()` / `renderer.computeAsync()` to execute.

## Quick Start

```javascript
import * as THREE from 'three/webgpu';
import { Fn, uniform, vec4, uv, sin, time } from 'three/tsl';
```

### Minimal TSL Shader

```javascript
const colorNode = Fn(() => {
  const uvCoord = uv();
  return vec4(uvCoord.x, sin(time).mul(0.5).add(0.5), uvCoord.y, 1.0);
})();

// Apply to material:
<meshStandardNodeMaterial colorNode={colorNode} />
```

### WebGPURenderer (React Three Fiber)

```jsx
<Canvas gl={async (props) => {
  const renderer = new THREE.WebGPURenderer(props);
  await renderer.init();
  return renderer;
}}>
```

## TSL Code Organization Pattern

Wrap TSL nodes in `useMemo`, separate `nodes` and `uniforms`:

```javascript
const { nodes, uniforms } = useMemo(() => {
  const time = uniform(0.0);
  const positionNode = Fn(() => { /* ... */ })();
  const colorNode = Fn(() => { /* ... */ })();
  return {
    nodes: { positionNode, colorNode },
    uniforms: { time },
  };
}, []);

useFrame(({ clock }) => { uniforms.time.value = clock.getElapsedTime(); });
```

## Common Gotchas

- **Textures are NOT uniforms**: Use `texture(myTex, uv())` for TSL, `texture(myTex)` + `sampler(tex)` for WGSL
- **WGSL texture sampling** requires both texture AND sampler: `textureSample(tex, samp, uv)`
- **`uniform()` types**: Only `boolean | number | Color | Vector2-4 | Matrix3-4`. NOT textures
- **Mutable variables**: Use `.toVar()` for reassignable values, then `.assign()` to update
- **`instanceIndex` is contextual**: In vertex stage = instance ID; in compute = thread ID
- **Post-processing render priority**: Use `useFrame(callback, 1)` to disable auto-render
- **R3F lights workaround**: `ambientLight`/`directionalLight` may need `scene.add()` instead of JSX
- **WebGPU max workgroup**: 256 threads per workgroup
- **Compute default workgroup**: `[64, 1, 1]`

## References

- **TSL API reference** (types, functions, nodes, materials): See [references/tsl-api.md](references/tsl-api.md)
- **Compute shaders** (buffers, workgroups, dispatch, atomics): See [references/compute-shaders.md](references/compute-shaders.md)
- **Recipes** (renderer setup, materials, particles, post-processing, GLSL porting): See [references/recipes.md](references/recipes.md)

## Tools

- TSL Transpiler (GLSL to TSL): `https://threejs.org/examples/webgpu_tsl_transpiler.html`
- TSL Editor: `https://threejs.org/examples/webgpu_tsl_editor.html`
