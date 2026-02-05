# TSL & WebGPU Recipes

## Table of Contents
1. [WebGPURenderer Setup (React Three Fiber)](#webgpurenderer-setup)
2. [WebGPURenderer Setup (Vanilla Three.js)](#vanilla-setup)
3. [Custom Material with Node System](#custom-material)
4. [Displacement + Normal Recomputation](#displacement-normals)
5. [Glass/Refraction Material](#glass-material)
6. [Particle System with Compute Shaders](#particles)
7. [Particle Attractors](#attractors)
8. [Post-Processing Setup (R3F)](#post-processing-r3f)
9. [Post-Processing Setup (Vanilla)](#post-processing-vanilla)
10. [Custom Post-Processing Effect](#custom-effect)
11. [Porting GLSL to TSL](#porting-glsl)

---

## WebGPURenderer Setup (React Three Fiber) {#webgpurenderer-setup}

```jsx
import * as THREE from 'three/webgpu';

const Scene = () => (
  <Canvas
    shadows
    gl={async (props) => {
      const renderer = new THREE.WebGPURenderer(props);
      await renderer.init();
      return renderer;
    }}
  >
    {/* scene contents */}
  </Canvas>
);

// Force WebGL fallback for testing:
const renderer = new THREE.WebGPURenderer({ ...props, forceWebGL: true });
```

## Vanilla Three.js Setup {#vanilla-setup}

```javascript
import * as THREE from 'three/webgpu';

const renderer = new THREE.WebGPURenderer({ antialias: true });
await renderer.init();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);
renderer.setAnimationLoop(animate);
```

## Custom Material with Node System {#custom-material}

Organize TSL nodes in `useMemo`, export `nodes` and `uniforms`:

```jsx
const { nodes, uniforms } = useMemo(() => {
  const time = uniform(0.0);
  const baseColor = uniform(new THREE.Color('#ff0000'));

  const colorNode = Fn(() => {
    const uvCoord = uv();
    const r = uvCoord.x.add(sin(time)).mul(0.5);
    return vec4(r, uvCoord.y, 0.5, 1.0);
  })();

  return {
    nodes: { colorNode },
    uniforms: { time },
  };
}, []);

useFrame(({ clock }) => {
  uniforms.time.value = clock.getElapsedTime();
});

return (
  <mesh>
    <sphereGeometry args={[1, 64, 64]} />
    <meshStandardNodeMaterial colorNode={nodes.colorNode} />
  </mesh>
);
```

## Displacement + Normal Recomputation {#displacement-normals}

```javascript
const { nodes, uniforms } = useMemo(() => {
  const time = uniform(0.0);
  const vNormal = varying(vec3(), 'vNormal');

  const displace = Fn(([pos, t]) => {
    const noise = mx_noise_float(pos.add(vec3(t)));
    return add(pos, normalLocal.mul(noise.mul(0.3)));
  });

  const orthogonal = Fn(() => {
    const n = normalLocal;
    If(abs(n.x).greaterThan(abs(n.z)), () => {
      return normalize(vec3(negate(n.y), n.x, 0.0));
    });
    return normalize(vec3(0.0, negate(n.z), n.y));
  });

  const positionNode = Fn(() => {
    const pos = positionLocal;
    const displaced = displace(pos, time);
    const theta = float(0.001);
    const tangent = orthogonal();
    const bitangent = normalize(cross(normalLocal, tangent));
    const n1 = displace(pos.add(tangent.mul(theta)), time);
    const n2 = displace(pos.add(bitangent.mul(theta)), time);
    const dTangent = n1.sub(displaced);
    const dBitangent = n2.sub(displaced);
    const normal = normalize(cross(dTangent, dBitangent));
    const corrected = normal.dot(normalLocal).lessThan(0.0)
      .select(normal.negate(), normal);
    vNormal.assign(corrected);
    return displaced;
  })();

  const normalNode = Fn(() => {
    return transformNormalToView(vNormal);
  })();

  return { nodes: { positionNode, normalNode }, uniforms: { time } };
}, []);
```

## Glass/Refraction Material {#glass-material}

Key pattern: sample scene texture at offset UVs per color channel for chromatic dispersion.

```javascript
const refractAndDisperse = Fn(({ sceneTex }) => {
  const LOOP = 8;
  const refractNormal = normalWorld.xy.mul(sub(1.0, normalWorld.z.mul(0.85)));
  const refractCol = vec3(0.0).toVar();

  for (let i = 0; i < LOOP; i++) {
    const slide = float(i).div(float(LOOP)).mul(0.18);
    const uvR = viewportUV.sub(refractNormal.mul(slide.mul(1.0).add(0.25)).mul(0.5));
    const uvG = viewportUV.sub(refractNormal.mul(slide.mul(2.5).add(0.25)).mul(0.5));
    const uvB = viewportUV.sub(refractNormal.mul(slide.mul(4.0).add(0.25)).mul(0.5));
    refractCol.assign(refractCol.add(vec3(
      texture(sceneTex, uvR).r,
      texture(sceneTex, uvG).g,
      texture(sceneTex, uvB).b,
    )));
  }
  refractCol.assign(refractCol.div(float(LOOP)));
  return vec3(refractCol);
});
```

## Particle System with Compute Shaders {#particles}

```jsx
const COUNT = 25000;
const { nodes } = useMemo(() => {
  const spawnBuffer = instancedArray(COUNT, 'vec3');
  const spawn = spawnBuffer.element(instanceIndex);

  const computeInit = Fn(() => {
    const idx = instanceIndex;
    const h0 = fract(sin(float(idx).mul(12.9898)).mul(43758.5453));
    const h1 = fract(sin(float(idx.add(1)).mul(12.9898)).mul(43758.5453));
    const h2 = fract(sin(float(idx.add(2)).mul(12.9898)).mul(43758.5453));
    const dist = sqrt(h0.mul(4.0));
    const theta = h1.mul(6.283);
    const phi = h2.mul(3.14159);
    spawn.assign(vec3(
      dist.mul(sin(phi)).mul(cos(theta)),
      dist.mul(sin(phi)).mul(sin(theta)),
      dist.mul(cos(phi)),
    ));
  })().compute(COUNT);

  const positionNode = Fn(() => spawn)();

  return { nodes: { computeInit, positionNode } };
}, []);

// Init once:
useEffect(() => { gl.computeAsync(nodes.computeInit); }, []);

return (
  <sprite count={COUNT}>
    <spriteNodeMaterial
      transparent depthWrite={false}
      blending={THREE.AdditiveBlending}
      positionNode={nodes.positionNode}
    />
  </sprite>
);
```

## Particle Attractors {#attractors}

```javascript
const spawnBuffer = instancedArray(COUNT, 'vec3');
const offsetBuffer = instancedArray(COUNT, 'vec3');
const spawn = spawnBuffer.element(instanceIndex);
const offset = offsetBuffer.element(instanceIndex);

const attractor = wgslFn(`
  fn thomas(pos: vec3f) -> vec3f {
    let b = 0.19; let dt = 0.015;
    return vec3f(
      (-b * pos.x + sin(pos.y)) * dt,
      (-b * pos.y + sin(pos.z)) * dt,
      (-b * pos.z + sin(pos.x)) * dt
    );
  }
`);

const computeUpdate = Fn(() => {
  const delta = attractor({ pos: spawn.add(offset) });
  offset.addAssign(delta);
})().compute(COUNT);

// Every frame:
useFrame(({ gl }) => gl.compute(computeUpdate));

const positionNode = Fn(() => spawn.add(offset))();
```

## Post-Processing Setup (React Three Fiber) {#post-processing-r3f}

```jsx
const postProcessingRef = useRef(null);

const { nodes } = useMemo(() => {
  const scenePass = pass(scene, camera);
  const diffuse = scenePass.getTextureNode('output');
  const outputNode = diffuse; // or add effects
  return { nodes: { outputNode } };
}, [scene, camera]);

useEffect(() => {
  const pp = new THREE.PostProcessing(gl);
  pp.outputNode = nodes.outputNode;
  postProcessingRef.current = pp;
  return () => { postProcessingRef.current = null; };
}, [gl, nodes.outputNode]);

// renderPriority=1 disables auto render, runs after other useFrames
useFrame(() => {
  postProcessingRef.current?.render();
}, 1);
```

## Post-Processing Setup (Vanilla) {#post-processing-vanilla}

```javascript
import { pass, bloom } from 'three/tsl';

const postProcessing = new THREE.PostProcessing(renderer);
const scenePass = pass(scene, camera);
const sceneColor = scenePass.getTextureNode('output');
const bloomPass = bloom(sceneColor);
postProcessing.outputNode = sceneColor.add(bloomPass);

function animate() {
  postProcessing.render();
}
renderer.setAnimationLoop(animate);
```

## Custom Post-Processing Effect {#custom-effect}

```javascript
class CustomEffectNode extends THREE.TempNode {
  constructor(inputNode) {
    super('vec4');
    this.inputNode = inputNode;
  }
  setup() {
    const input = this.inputNode;
    const effect = Fn(() => {
      const color = input;
      // Example: vignette
      const dist = distance(viewportUV, vec2(0.5));
      const vignette = smoothstep(0.8, 0.2, dist);
      return vec4(color.rgb.mul(vignette), 1.0);
    });
    return effect();
  }
}
const customPass = (node) => nodeObject(new CustomEffectNode(node));

// Usage:
postProcessing.outputNode = customPass(sceneColor);
```

## MRT (Multiple Render Targets) {#mrt}

```javascript
const scenePass = pass(scene, camera);
scenePass.setMRT(mrt({
  output: output,
  normal: normalView,
}));
const normalTex = scenePass.getTextureNode('normal');
const diffuseTex = scenePass.getTextureNode('output');
```

## Porting GLSL to TSL {#porting-glsl}

| GLSL | TSL |
|------|-----|
| `float x = 1.0;` | `const x = float(1.0);` or `float(1.0).toVar()` for mutable |
| `vec3 v = vec3(1.0);` | `const v = vec3(1.0);` |
| `a + b` | `add(a, b)` or `a.add(b)` |
| `a * b` | `mul(a, b)` or `a.mul(b)` |
| `mix(a, b, t)` | `mix(a, b, t)` |
| `texture2D(tex, uv)` | `texture(tex, uv())` |
| `gl_FragCoord` | `viewportUV` (normalized) |
| `gl_Position` | `positionNode` on material |
| `uniform float time;` | `const time = uniform(0.0);` |
| `varying vec3 vNorm;` | `const vNorm = varying(vec3(), 'vNorm');` |
| `if (x > 0.0)` | `If(x.greaterThan(0.0), () => { ... })` |
| `x > 0.0 ? a : b` | `x.greaterThan(0.0).select(a, b)` |
| `for (int i...)` | `Loop(count, ({ i }) => { ... })` |

Use the TSL transpiler: https://threejs.org/examples/webgpu_tsl_transpiler.html
Use the TSL editor: https://threejs.org/examples/webgpu_tsl_editor.html
