const TARGET_BY_PLATFORM = new Map([
  ["linux:x64", "x86_64-unknown-linux-musl"],
  ["linux:arm64", "aarch64-unknown-linux-musl"],
  ["darwin:x64", "x86_64-apple-darwin"],
  ["darwin:arm64", "aarch64-apple-darwin"],
  ["win32:x64", "x86_64-pc-windows-msvc"],
]);

export function resolveTarget(platform, arch) {
  return TARGET_BY_PLATFORM.get(`${platform}:${arch}`) ?? null;
}
