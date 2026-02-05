# Particles & GPU Simulation

## Table of Contents
- [Basic Particle System](#basic-particle-system)
- [Buffer Geometry Attributes](#buffer-geometry-attributes)
- [Particle Vertex/Fragment Shaders](#particle-vertexfragment-shaders)
- [FBO / GPGPU Simulation](#fbo--gpgpu-simulation)
- [Instanced Particles](#instanced-particles)
- [R3F Patterns](#r3f-patterns)

## Basic Particle System

Particles in Three.js/R3F are `THREE.Points` with custom `BufferGeometry` and `ShaderMaterial`.

```tsx
function Particles({ count = 5000 }) {
  const { positions, randoms } = useMemo(() => {
    const positions = new Float32Array(count * 3)
    const randoms = new Float32Array(count)
    for (let i = 0; i < count; i++) {
      positions[i * 3]     = (Math.random() - 0.5) * 10
      positions[i * 3 + 1] = (Math.random() - 0.5) * 10
      positions[i * 3 + 2] = (Math.random() - 0.5) * 10
      randoms[i] = Math.random()
    }
    return { positions, randoms }
  }, [count])

  return (
    <points>
      <bufferGeometry>
        <bufferAttribute attach="attributes-position" args={[positions, 3]} />
        <bufferAttribute attach="attributes-aRandom" args={[randoms, 1]} />
      </bufferGeometry>
      <shaderMaterial
        vertexShader={vertexShader}
        fragmentShader={fragmentShader}
        transparent
        depthWrite={false}
        blending={THREE.AdditiveBlending}
      />
    </points>
  )
}
```

## Buffer Geometry Attributes

Custom per-particle data passed as **attributes** (per-vertex data available in vertex shader).

| Attribute | Item Size | Purpose |
|-----------|-----------|---------|
| `position` | 3 | xyz coordinates |
| `aRandom` | 1 | random seed per particle |
| `aColor` | 3 | per-particle color |
| `aSize` | 1 | per-particle size |
| `aVelocity` | 3 | per-particle velocity |
| `aLife` | 1 | particle lifetime |

```tsx
<bufferAttribute
  attach="attributes-aColor"
  args={[colorArray, 3]}
/>
```

In vertex shader: `attribute float aRandom;` (WebGL) or accessed via the attribute name.

## Particle Vertex/Fragment Shaders

### Vertex shader (point-based)
```glsl
uniform float uTime;
uniform float uPixelRatio;
uniform float uSize;
attribute float aRandom;
attribute float aSize;
varying float vRandom;

void main() {
  vRandom = aRandom;
  vec3 pos = position;

  // Animate
  pos.y += sin(uTime + aRandom * 6.28) * 0.3;

  vec4 mvPosition = modelViewMatrix * vec4(pos, 1.0);

  // Size attenuation (smaller when farther)
  gl_PointSize = uSize * aSize * uPixelRatio * (1.0 / -mvPosition.z);
  gl_Position = projectionMatrix * mvPosition;
}
```

### Fragment shader (circular particles)
```glsl
varying float vRandom;
uniform vec3 uColor;

void main() {
  // Circular shape
  float dist = length(gl_PointCoord - vec2(0.5));
  if (dist > 0.5) discard;

  // Soft edge
  float alpha = 1.0 - smoothstep(0.3, 0.5, dist);
  vec3 color = uColor * (0.5 + vRandom * 0.5);
  gl_FragColor = vec4(color, alpha);
}
```

## FBO / GPGPU Simulation

Use Frame Buffer Objects to run simulation on the GPU. Each pixel in a texture stores one particle's data (position, velocity, etc.).

### Concept

1. **Data texture**: Encode particle positions/velocities as RGBA pixels in a texture
2. **Simulation shader**: Read previous frame's texture, compute new positions, write to new texture
3. **Render shader**: Read position texture, render particles at those positions
4. **Ping-pong**: Alternate between two FBOs each frame (read from one, write to other)

### R3F with useFBO

```tsx
import { useFBO } from '@react-three/drei'
import { createPortal, useFrame } from '@react-three/fiber'
import * as THREE from 'three'

function GPGPUParticles({ count = 256 }) {
  const size = count // texture is size x size = count^2 particles

  // Create data texture with initial positions
  const data = useMemo(() => {
    const d = new Float32Array(size * size * 4)
    for (let i = 0; i < size * size; i++) {
      d[i * 4]     = (Math.random() - 0.5) * 10 // x
      d[i * 4 + 1] = (Math.random() - 0.5) * 10 // y
      d[i * 4 + 2] = (Math.random() - 0.5) * 10 // z
      d[i * 4 + 3] = 1.0                          // w (life)
    }
    const tex = new THREE.DataTexture(d, size, size, THREE.RGBAFormat, THREE.FloatType)
    tex.needsUpdate = true
    return tex
  }, [size])

  // Two FBOs for ping-pong
  const fboA = useFBO(size, size, {
    minFilter: THREE.NearestFilter,
    magFilter: THREE.NearestFilter,
    format: THREE.RGBAFormat,
    type: THREE.FloatType,
  })
  const fboB = useFBO(size, size, { /* same options */ })

  // Simulation material reads one FBO, writes to the other
  // Render pass reads the result FBO to position particles
}
```

### Simulation shader (fragment)
```glsl
uniform sampler2D uPositions; // previous frame positions
uniform float uTime;
uniform float uDelta;
varying vec2 vUv;

void main() {
  vec4 pos = texture2D(uPositions, vUv);

  // Simple curl noise velocity
  vec3 vel = curlNoise(pos.xyz * 0.5 + uTime * 0.1);
  pos.xyz += vel * uDelta * 0.5;

  // Respawn if too far
  if (length(pos.xyz) > 5.0) {
    pos.xyz = vec3(0.0);
  }

  gl_FragColor = pos;
}
```

### Reading positions in render vertex shader
```glsl
uniform sampler2D uPositionTexture;
attribute vec2 aReference; // UV coordinates into position texture

void main() {
  vec3 pos = texture2D(uPositionTexture, aReference).xyz;
  vec4 mvPosition = modelViewMatrix * vec4(pos, 1.0);
  gl_PointSize = 4.0 * (1.0 / -mvPosition.z);
  gl_Position = projectionMatrix * mvPosition;
}
```

## Instanced Particles

For complex per-particle geometry (not just points), use `InstancedMesh`.

```tsx
function InstancedParticles({ count = 1000 }) {
  const meshRef = useRef()
  const dummy = useMemo(() => new THREE.Object3D(), [])

  useFrame((state) => {
    for (let i = 0; i < count; i++) {
      dummy.position.set(
        Math.sin(state.clock.elapsedTime + i) * 3,
        Math.cos(state.clock.elapsedTime + i * 0.5) * 2,
        i * 0.01
      )
      dummy.updateMatrix()
      meshRef.current.setMatrixAt(i, dummy.matrix)
    }
    meshRef.current.instanceMatrix.needsUpdate = true
  })

  return (
    <instancedMesh ref={meshRef} args={[null, null, count]}>
      <sphereGeometry args={[0.05, 8, 8]} />
      <meshStandardMaterial color="hotpink" />
    </instancedMesh>
  )
}
```

For GPU-driven instancing, pass instance data via `InstancedBufferAttribute` and process in vertex shader.

## R3F Patterns

### useFrame for animation
```tsx
useFrame((state, delta) => {
  materialRef.current.uniforms.uTime.value = state.clock.elapsedTime
  // Use delta for frame-rate independent animation
})
```

### Additive blending for glowing particles
```tsx
<shaderMaterial
  transparent
  depthWrite={false}
  blending={THREE.AdditiveBlending}
/>
```

### Size attenuation
Scale `gl_PointSize` by `1.0 / -mvPosition.z` to make particles smaller when farther from camera. Multiply by `uPixelRatio` (pass `window.devicePixelRatio`) for consistent size across displays.
