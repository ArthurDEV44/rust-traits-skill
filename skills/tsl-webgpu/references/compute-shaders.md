# Compute Shaders with TSL

## Overview

Compute shaders run on the GPU's compute pipeline (no vertices, no rasterization, no fragments). They take arbitrary input data, execute workgroups of threads, and output results to buffers or textures.

## Basic Pattern

```javascript
import { Fn, instanceIndex, instancedArray, uniform, wgslFn, code } from 'three/tsl';

// 1. Create storage buffer
const buffer = instancedArray(COUNT, 'vec3');

// 2. Define compute shader
const computeFn = Fn(() => {
  const idx = instanceIndex;
  const pos = buffer.element(idx);
  pos.assign(vec3(float(idx).div(COUNT), 0.0, 0.0));
})().compute(COUNT);

// 3. Execute
await renderer.computeAsync(computeFn);
// or synchronously:
renderer.compute(computeFn);
```

## WGSL Compute Shader

```javascript
const hash = code(`
  fn hash(index: u32) -> f32 {
    return fract(sin(f32(index) * 12.9898) * 43758.5453);
  }
`);

const computeInit = wgslFn(`
  fn compute(
    buffer: ptr<storage, array<vec3f>, read_write>,
    count: f32,
    index: u32,
  ) -> void {
    let h = hash(index);
    buffer[index] = vec3f(h, 0.0, 0.0);
  }
`, [hash]);

const computeNode = computeInit({
  buffer: buffer,
  count: COUNT,
  index: instanceIndex,
}).compute(COUNT);
```

## Workgroup and Dispatch Sizes

```javascript
// Default workgroup: [64, 1, 1]
// Custom workgroup:
const node = computeFn().compute(COUNT, [64, 1, 1]);

// Manual dispatch with computeKernel:
const kernel = computeFn().computeKernel([8, 8, 1]);
renderer.compute(kernel, [4, 4, 1]);

// Total threads = workgroupSize * dispatchSize per dimension
// Example: [64,1,1] workgroup + [16,1,1] dispatch = 1024 threads
// Example: [4,4,1] workgroup + [8,8,1] dispatch = 32*32 = 1024 threads
```

Max 256 threads per workgroup.

## Instanced Meshes with Compute

```javascript
const COUNT = 1024;
const posBuffer = instancedArray(COUNT, 'vec3');

// Init positions via compute
const computeInit = Fn(() => {
  const idx = instanceIndex;
  const x = float(idx.mod(32)).mul(0.6).sub(9.3);
  const z = float(idx.div(32)).mul(0.6).sub(9.3);
  posBuffer.element(idx).assign(vec3(x, 0.0, z));
})().compute(COUNT);

// Use in render pipeline
const positionNode = positionLocal.add(posBuffer.element(instanceIndex));

// JSX
<instancedMesh args={[undefined, undefined, COUNT]}>
  <icosahedronGeometry args={[0.3, 16]} />
  <meshPhongNodeMaterial positionNode={positionNode} />
</instancedMesh>
```

## Updating Every Frame

```javascript
const time = uniform(0.0);

const computeUpdate = Fn(() => {
  const idx = instanceIndex;
  const pos = posBuffer.element(idx);
  const offset = sin(pos.x.add(time)).mul(0.5);
  pos.y.assign(offset);
})().compute(COUNT);

// In animation loop:
useFrame((state) => {
  time.value = state.clock.getElapsedTime();
  state.gl.compute(computeUpdate);
});
```

## Reading Back Data to CPU

```javascript
const responseBuffer = storage(new Float32Array(16), 'mat4x4f', 1);

// After compute:
await renderer.computeAsync(computeNode);
const output = new Float32Array(
  await renderer.getArrayBufferAsync(responseBuffer)
);
```

## StorageTexture

```javascript
const storageTex = new THREE.StorageTexture(
  window.innerWidth * window.devicePixelRatio,
  window.innerHeight * window.devicePixelRatio
);

// In compute shader:
const posX = instanceIndex.mod(int(width));
const posY = instanceIndex.div(width);
const fragCoord = uvec2(posX, posY);
textureStore(storageTex, fragCoord, vec4(value, 0.0, 0.0, 1.0));

// Total threads = width * height
computeFn().compute(width * height);
```

## Atomic Operations

```javascript
import { atomicAdd, atomicMax, atomicStore } from 'three/tsl';

atomicAdd(buffer.element(idx), float(1.0));
atomicMax(buffer.element(idx), value);
atomicStore(buffer.element(idx), value);
```

## Barrier

```javascript
import { barrier } from 'three/tsl';
barrier('storage'); // Wait for storage writes to complete
```
