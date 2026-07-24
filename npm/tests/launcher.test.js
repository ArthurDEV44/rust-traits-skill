import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import {
  copyFile,
  mkdir,
  mkdtemp,
  rm,
  symlink,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { resolveTarget } from "../lib/target.js";

const sourceRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
);

async function prepareLauncher() {
  const target = resolveTarget(process.platform, process.arch);
  assert.notEqual(target, null);

  const packageRoot = await mkdtemp(path.join(tmpdir(), "arthur-skills-launcher-"));
  const binRoot = path.join(packageRoot, "bin");
  const libRoot = path.join(packageRoot, "lib");
  const vendorRoot = path.join(packageRoot, "vendor", target);
  await mkdir(binRoot, { recursive: true });
  await mkdir(libRoot, { recursive: true });
  await mkdir(vendorRoot, { recursive: true });

  const launcher = path.join(binRoot, "arthur-skills.js");
  await copyFile(path.join(sourceRoot, "bin", "arthur-skills.js"), launcher);
  await copyFile(
    path.join(sourceRoot, "lib", "target.js"),
    path.join(libRoot, "target.js"),
  );
  await symlink(process.execPath, path.join(vendorRoot, "arthur-skills"));

  return { launcher, packageRoot };
}

function waitForExit(child) {
  return new Promise((resolve, reject) => {
    child.once("error", reject);
    child.once("exit", (exitCode, signal) => resolve({ exitCode, signal }));
  });
}

test(
  "forwards the native exit code",
  { skip: process.platform === "win32" },
  async () => {
    const { launcher, packageRoot } = await prepareLauncher();
    try {
      const child = spawn(process.execPath, [
        launcher,
        "-e",
        "process.exit(7)",
      ]);
      assert.deepEqual(await waitForExit(child), {
        exitCode: 7,
        signal: null,
      });
    } finally {
      await rm(packageRoot, { recursive: true, force: true });
    }
  },
);

test(
  "forwards and preserves a termination signal",
  { skip: process.platform === "win32" },
  async () => {
    const { launcher, packageRoot } = await prepareLauncher();
    try {
      const child = spawn(process.execPath, [
        launcher,
        "-e",
        "setInterval(() => {}, 1_000)",
      ]);
      const exit = waitForExit(child);
      await new Promise((resolve) => setTimeout(resolve, 100));
      child.kill("SIGTERM");
      assert.deepEqual(await exit, {
        exitCode: null,
        signal: "SIGTERM",
      });
    } finally {
      await rm(packageRoot, { recursive: true, force: true });
    }
  },
);
