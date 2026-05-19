import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import { mkdir, readdir, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import { spawnSync } from "node:child_process";

const args = new Map(
  process.argv
    .slice(2)
    .filter((arg) => arg.startsWith("--"))
    .map((arg) => {
      const [key, value = "true"] = arg.slice(2).split("=");
      return [key, value];
    }),
);
const platform = normalizePlatform(
  args.get("platform") ??
    process.env.AXIOM_RELEASE_PLATFORM ??
    process.env.RUNNER_OS ??
    process.platform,
);
const mode = args.get("mode") ?? process.env.AXIOM_RELEASE_MODE ?? "dry-run";
const bundleDir = args.get("bundle-dir") ?? "src-tauri/target/release/bundle";
const manifestDir = args.get("manifest-dir") ?? ".release";
const signed = mode === "signed" || process.env.GITHUB_REF_TYPE === "tag";
const artifactFiles = (await walk(bundleDir)).filter((file) =>
  artifactMatchesPlatform(file, platform),
);

if (artifactFiles.length === 0) {
  throw new Error(`No ${platform} release artifacts were found under ${bundleDir}.`);
}

await mkdir(manifestDir, { recursive: true });
const artifacts = [];

for (const file of artifactFiles) {
  const fileStat = await stat(file);
  artifacts.push({
    path: file,
    sizeBytes: fileStat.size,
    sha256: await sha256File(file),
  });
}

if (signed) {
  verifySignedArtifacts(platform, artifactFiles);
}

const manifest = {
  generatedAt: new Date().toISOString(),
  platform,
  mode,
  signed,
  artifacts,
};
const manifestPath = path.join(manifestDir, `axiomphp-release-${platform}.json`);

await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(JSON.stringify({ manifestPath, artifactCount: artifacts.length }, null, 2));

async function walk(root) {
  const entries = await readdir(root, { withFileTypes: true });
  const files = [];

  for (const entry of entries) {
    const current = path.join(root, entry.name);

    if (entry.isDirectory()) {
      files.push(...(await walk(current)));
    } else if (entry.isFile()) {
      files.push(current);
    }
  }

  return files;
}

function artifactMatchesPlatform(file, targetPlatform) {
  const lower = file.toLowerCase();

  if (targetPlatform === "macos") {
    return lower.endsWith(".dmg") || lower.endsWith(".app.tar.gz");
  }

  if (targetPlatform === "windows") {
    return lower.endsWith(".msi") || lower.endsWith(".exe") || lower.endsWith(".msix");
  }

  return !lower.endsWith(".sig") && !lower.endsWith(".json");
}

async function sha256File(file) {
  const hash = createHash("sha256");
  await new Promise((resolve, reject) => {
    createReadStream(file)
      .on("data", (chunk) => hash.update(chunk))
      .on("error", reject)
      .on("end", resolve);
  });
  return hash.digest("hex");
}

function verifySignedArtifacts(targetPlatform, files) {
  if (targetPlatform === "windows" && process.platform === "win32") {
    for (const file of files.filter((item) => /\.(exe|msi|msix)$/i.test(item))) {
      const result = spawnSync(
        "powershell",
        [
          "-NoProfile",
          "-Command",
          `$sig = Get-AuthenticodeSignature -LiteralPath '${escapePowerShell(file)}'; if ($sig.Status -ne 'Valid') { Write-Error $sig.Status; exit 1 }`,
        ],
        { stdio: "inherit" },
      );

      if (result.status !== 0) {
        throw new Error(`Windows signature verification failed for ${file}.`);
      }
    }
  }

  if (targetPlatform === "macos" && process.platform === "darwin") {
    const appBundles = files.filter((item) => item.endsWith(".app"));

    for (const appBundle of appBundles) {
      const result = spawnSync("codesign", ["--verify", "--deep", "--strict", appBundle], {
        stdio: "inherit",
      });

      if (result.status !== 0) {
        throw new Error(`macOS code signature verification failed for ${appBundle}.`);
      }
    }
  }
}

function escapePowerShell(value) {
  return value.replaceAll("'", "''");
}

function normalizePlatform(value) {
  const lower = String(value ?? "").toLowerCase();

  if (lower === "darwin" || lower === "macos" || lower === "macos-latest") {
    return "macos";
  }

  if (lower === "win32" || lower === "windows" || lower === "windows-latest") {
    return "windows";
  }

  return lower || "unknown";
}
