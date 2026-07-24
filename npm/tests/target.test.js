import assert from "node:assert/strict";
import test from "node:test";

import { resolveTarget } from "../lib/target.js";

test("resolves every published native target", () => {
  assert.deepEqual(
    [
      ["linux", "x64"],
      ["linux", "arm64"],
      ["darwin", "x64"],
      ["darwin", "arm64"],
      ["win32", "x64"],
    ].map(([platform, arch]) => resolveTarget(platform, arch)),
    [
      "x86_64-unknown-linux-musl",
      "aarch64-unknown-linux-musl",
      "x86_64-apple-darwin",
      "aarch64-apple-darwin",
      "x86_64-pc-windows-msvc",
    ],
  );
});

test("rejects targets without a published binary", () => {
  assert.equal(resolveTarget("win32", "arm64"), null);
  assert.equal(resolveTarget("freebsd", "x64"), null);
  assert.equal(resolveTarget("linux", "riscv64"), null);
});
