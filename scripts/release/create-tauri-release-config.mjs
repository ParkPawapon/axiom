import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const args = new Map(
  process.argv
    .slice(2)
    .filter((arg) => arg.startsWith("--"))
    .map((arg) => {
      const [key, value = "true"] = arg.slice(2).split("=");
      return [key, value];
    }),
);
const mode = args.get("mode") ?? process.env.AXIOM_RELEASE_MODE ?? "dry-run";
const platform = normalizePlatform(
  args.get("platform") ??
    process.env.AXIOM_RELEASE_PLATFORM ??
    process.env.RUNNER_OS ??
    process.platform,
);
const inputPath = args.get("input") ?? "src-tauri/tauri.conf.json";
const outputPath = args.get("output") ?? "src-tauri/tauri.release.generated.conf.json";
const signed = mode === "signed" || process.env.GITHUB_REF_TYPE === "tag";
const config = JSON.parse(await readFile(inputPath, "utf8"));

config.bundle = config.bundle ?? {};

if (platform === "macos" || platform === "all") {
  config.bundle.macOS = {
    ...(config.bundle.macOS ?? {}),
    entitlements: "entitlements/macos.plist",
    hardenedRuntime: true,
  };

  if (signed && process.env.APPLE_SIGNING_IDENTITY) {
    config.bundle.macOS.signingIdentity = process.env.APPLE_SIGNING_IDENTITY;
  } else if (!signed) {
    config.bundle.macOS.signingIdentity = "-";
  }
}

if (platform === "windows" || platform === "all") {
  config.bundle.windows = {
    ...(config.bundle.windows ?? {}),
    digestAlgorithm: process.env.WINDOWS_DIGEST_ALGORITHM || "sha256",
    timestampUrl: process.env.WINDOWS_TIMESTAMP_URL || "http://timestamp.digicert.com",
  };

  if (signed && process.env.WINDOWS_CERTIFICATE_THUMBPRINT) {
    config.bundle.windows.certificateThumbprint = process.env.WINDOWS_CERTIFICATE_THUMBPRINT;
  }

  if (signed && process.env.WINDOWS_SIGN_COMMAND) {
    config.bundle.windows.signCommand = process.env.WINDOWS_SIGN_COMMAND;
  }
}

await mkdir(path.dirname(outputPath), { recursive: true });
await writeFile(outputPath, `${JSON.stringify(config, null, 2)}\n`);

console.log(
  JSON.stringify(
    {
      outputPath,
      mode,
      platform,
      signed,
      configured: {
        macOS: Boolean(config.bundle.macOS),
        windows: Boolean(config.bundle.windows),
      },
    },
    null,
    2,
  ),
);

function normalizePlatform(value) {
  const lower = String(value ?? "").toLowerCase();

  if (lower === "darwin" || lower === "macos" || lower === "macos-latest") {
    return "macos";
  }

  if (lower === "win32" || lower === "windows" || lower === "windows-latest") {
    return "windows";
  }

  if (lower === "all") {
    return "all";
  }

  return lower || "unknown";
}
