#!/usr/bin/env node

import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { resolveTarget } from "../lib/target.js";

const target = resolveTarget(process.platform, process.arch);
if (target === null) {
  console.error(
    `@arthjean/skills does not support ${process.platform} (${process.arch}).`,
  );
  process.exit(1);
}

const packageRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
);
const binaryName =
  process.platform === "win32" ? "arthur-skills.exe" : "arthur-skills";
const binaryPath = path.join(packageRoot, "vendor", target, binaryName);

if (!existsSync(binaryPath)) {
  console.error(
    `Native binary missing for ${target}. Reinstall @arthjean/skills@latest with your package manager.`,
  );
  process.exit(1);
}

const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: "inherit",
  env: process.env,
});

child.on("error", (error) => {
  console.error(error);
  process.exit(1);
});

const forwardSignal = (signal) => {
  if (child.killed) {
    return;
  }

  try {
    child.kill(signal);
  } catch {
    // The child may have exited between the killed check and signal delivery.
  }
};

const signalHandlers = new Map();
for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) {
  const handler = () => forwardSignal(signal);
  signalHandlers.set(signal, handler);
  process.on(signal, handler);
}

const result = await new Promise((resolve) => {
  child.on("exit", (exitCode, signal) => {
    if (signal !== null) {
      resolve({ signal });
    } else {
      resolve({ exitCode: exitCode ?? 1 });
    }
  });
});

if ("signal" in result) {
  for (const [signal, handler] of signalHandlers) {
    process.off(signal, handler);
  }
  process.kill(process.pid, result.signal);
} else {
  process.exit(result.exitCode);
}
