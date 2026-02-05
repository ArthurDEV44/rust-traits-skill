# TSL API Reference

## Imports

```javascript
import * as THREE from 'three/webgpu';
import { Fn, uniform, varying, texture, sampler, uv, vec2, vec3, vec4, float, int,
  positionLocal, positionWorld, normalLocal, normalWorld, cameraPosition,
  viewportUV, instanceIndex, storage, instancedArray, pass, time,
  mix, add, sub, mul, div, mod, abs, dot, cross, normalize, pow, sin, cos,
  If, Loop, textureStore, code, wgslFn, glslFn } from 'three/tsl';
import { bloom } from 'three/addons/tsl/display/BloomNode.js';
```

`three/webgpu` = THREE namespace (WebGPURenderer, materials, geometries, etc.)
`three/tsl` = All TSL shading functions and nodes.

## Type Constructors & Conversions

```
float(x), int(x), uint(x), bool(x)
vec2(x,y), vec3(x,y,z), vec4(x,y,z,w)
ivec2, ivec3, ivec4, uvec2, uvec3, uvec4
bvec2, bvec3, bvec4
mat2, mat3, mat4
color(r,g,b)
```

Chainable conversions: `node.toFloat()`, `node.toVec3()`, `node.toInt()`, etc.
Automatic conversions between compatible types.

## Core Functions

### Fn - Define TSL shader functions

```javascript
const myShader = Fn(([arg1, arg2]) => { /* return node */ });
const myShader = Fn(({ namedArg1, namedArg2 }) => { /* return node */ });
// Invoke immediately for node assignment:
const node = Fn(() => { /* ... */ })();
```

### uniform - Declare uniforms

```javascript
const time = uniform(0.0);        // float
const col = uniform(new THREE.Color(1,0,0)); // color
const pos = uniform(new THREE.Vector3());    // vec3
// Update: time.value = newValue;
```

Supported types: `boolean | number | Color | Vector2 | Vector3 | Vector4 | Matrix3 | Matrix4`

Update callbacks:
- `uniform.onObjectUpdate(({ object }) => value)` - per object render
- `uniform.onRenderUpdate(() => value)` - once per render
- `uniform.onFrameUpdate(() => value)` - once per frame

### varying - Pass data vertex -> fragment

```javascript
const vNormal = varying(vec3(), 'vNormal');
// In vertex: vNormal.assign(computedNormal);
// In fragment: use vNormal directly
```

### texture / sampler - Texture operations

```javascript
// TSL direct sampling:
const texel = texture(myTexture, uv());

// For WGSL shaders:
const tex = texture(myTexture);
const samp = sampler(tex);
// Pass tex and samp as arguments to wgslFn
// In WGSL: textureSample(sceneTex, sampler, uvCoord)
```

### storage / instancedArray - GPU buffers

```javascript
const buffer = instancedArray(COUNT, 'vec3');    // per-instance
const buf = storage(typedArray, 'vec3', count);  // generic storage
// Read: buffer.element(instanceIndex)
// Write (compute only): buffer.element(idx).assign(value)
// Convert to vertex attribute: buffer.toAttribute()
```

### textureStore - Write to StorageTexture

```javascript
const storageTex = new THREE.StorageTexture(width, height);
textureStore(storageTex, uvec2(x, y), vec4(r, g, b, a));
```

## Built-in Accessors

| Accessor | Type | Description |
|----------|------|-------------|
| `positionLocal` | vec3 | Object-space vertex position |
| `positionWorld` | vec3 | World-space vertex position |
| `normalLocal` | vec3 | Object-space normal |
| `normalWorld` | vec3 | World-space normal |
| `uv()` | vec2 | Texture coordinates |
| `viewportUV` | vec2 | Screen-space UV (0-1) |
| `cameraPosition` | vec3 | Camera world position |
| `instanceIndex` | uint | Instance/thread ID |
| `time` | float | Elapsed time (auto-updated) |

## Math Functions

**Arithmetic**: `add`, `sub`, `mul`, `div`, `mod`, `negate`, `oneMinus`
**Trig**: `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`
**Exponential**: `exp`, `exp2`, `log`, `log2`, `pow`, `sqrt`, `inverseSqrt`
**Interpolation**: `mix`, `smoothstep`, `step`, `clamp`
**Comparison**: `min`, `max`, `abs`, `sign`, `floor`, `ceil`, `fract`, `round`
**Vector**: `length`, `distance`, `dot`, `cross`, `normalize`, `reflect`, `refract`, `faceForward`
**Matrix**: `determinant`, `inverse`, `transpose`

Chaining: `node.add(x)`, `node.mul(x)`, `node.sub(x)`, `node.div(x)`, `node.mod(x)`

## Control Flow

```javascript
If(condition, () => { /* then */ });
If(condition.greaterThan(x), () => { /* then */ });

// Ternary: condition.select(trueVal, falseVal)
node.lessThan(0.0).select(node.negate(), node);

Loop(count, ({ i }) => { /* body */ });
```

## Native Shader Code

### wgslFn - Write raw WGSL

```javascript
const myFn = wgslFn(`
  fn myFunc(pos: vec3f, time: f32) -> vec3f {
    return pos + vec3f(sin(time), 0.0, 0.0);
  }
`);
// Use: myFn({ pos: positionLocal, time: timeUniform })
```

### glslFn - Write raw GLSL

```javascript
const myFn = glslFn(`
  vec4 myFunc(vec4 color, vec2 uv) {
    return mix(color, vec4(1.0), uv.x);
  }
`);
```

### code - Reusable WGSL helper functions

```javascript
const helpers = code(`
  fn hash(seed: u32) -> f32 {
    return fract(sin(f32(seed) * 12.9898) * 43758.5453);
  }
`);
const main = wgslFn(`fn main(...) { let h = hash(0u); }`, [helpers]);
```

## Node Materials

All stock materials have node equivalents:

| Classic | Node Material |
|---------|--------------|
| MeshStandardMaterial | MeshStandardNodeMaterial |
| MeshPhysicalMaterial | MeshPhysicalNodeMaterial |
| MeshPhongMaterial | MeshPhongNodeMaterial |
| MeshBasicMaterial | MeshBasicNodeMaterial |
| SpriteNodeMaterial | SpriteNodeMaterial |

### Key Node Properties

| Property | Stage | Purpose |
|----------|-------|---------|
| `positionNode` | Vertex | Override vertex positions |
| `normalNode` | Vertex | Override normals |
| `colorNode` | Fragment | Override color output |
| `opacityNode` | Fragment | Override opacity |
| `emissiveNode` | Fragment | Override emissive |
| `roughnessNode` | Fragment | Override roughness |
| `metalnessNode` | Fragment | Override metalness |

```javascript
<meshPhongNodeMaterial
  positionNode={nodes.positionNode}
  normalNode={nodes.normalNode}
  colorNode={nodes.colorNode}
/>
```

## Noise Functions (built-in)

```javascript
import { mx_noise_float, mx_noise_vec3 } from 'three/tsl';
// mx_noise_float(vec3) -> float (Perlin noise)
// mx_noise_vec3(vec3) -> vec3
```

## Post-Processing Functions

```javascript
// From three/tsl:
pass, mrt, bloom, fxaa, gaussianBlur, dotScreen, film, dof, ao
// Usage:
const scenePass = pass(scene, camera);
const diffuse = scenePass.getTextureNode('output');
const depth = scenePass.getTextureNode('depth');
```

## Variable Declaration

```javascript
const myVar = float(0.0).toVar();   // mutable variable
myVar.assign(newValue);              // reassign
myVar.addAssign(delta);              // +=
myVar.mulAssign(factor);             // *=
```
