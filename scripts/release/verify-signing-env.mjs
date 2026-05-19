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
const signed = mode === "signed" || process.env.GITHUB_REF_TYPE === "tag";
const checks = platform === "all" ? ["macos", "windows"] : [platform];
const missing = [];
const warnings = [];

for (const check of checks) {
  if (check === "macos") {
    verifyMacos();
  } else if (check === "windows") {
    verifyWindows();
  } else {
    warnings.push(`No release signing requirements are defined for platform ${check}.`);
  }
}

const result = {
  mode,
  platform,
  signed,
  missing,
  warnings,
  status: missing.length === 0 ? "ready" : signed ? "blocked" : "advisory",
};

console.log(JSON.stringify(result, null, 2));

if (signed && missing.length > 0) {
  process.exitCode = 1;
}

function verifyMacos() {
  requireAll("macOS signing", [
    "APPLE_CERTIFICATE",
    "APPLE_CERTIFICATE_PASSWORD",
    "APPLE_SIGNING_IDENTITY",
  ]);

  const hasAppStoreConnectApi =
    has("APPLE_API_ISSUER") &&
    has("APPLE_API_KEY") &&
    (has("APPLE_API_KEY_PATH") || has("APPLE_API_KEY_BASE64"));
  const hasAppleId = has("APPLE_ID") && has("APPLE_PASSWORD") && has("APPLE_TEAM_ID");

  if (!hasAppStoreConnectApi && !hasAppleId) {
    missing.push(
      "macOS notarization requires APPLE_API_ISSUER + APPLE_API_KEY + APPLE_API_KEY_PATH/APPLE_API_KEY_BASE64, or APPLE_ID + APPLE_PASSWORD + APPLE_TEAM_ID",
    );
  }
}

function verifyWindows() {
  const hasPfxCertificate =
    has("WINDOWS_CERTIFICATE") &&
    has("WINDOWS_CERTIFICATE_PASSWORD") &&
    has("WINDOWS_CERTIFICATE_THUMBPRINT");
  const hasAzureTrustedSigning =
    has("AZURE_CLIENT_ID") &&
    has("AZURE_CLIENT_SECRET") &&
    has("AZURE_TENANT_ID") &&
    has("AZURE_TRUSTED_SIGNING_ACCOUNT") &&
    has("AZURE_TRUSTED_SIGNING_CERTIFICATE_PROFILE") &&
    has("AZURE_TRUSTED_SIGNING_ENDPOINT");
  const hasCustomSignCommand = has("WINDOWS_SIGN_COMMAND");

  if (!hasPfxCertificate && !hasAzureTrustedSigning && !hasCustomSignCommand) {
    missing.push(
      "Windows signing requires WINDOWS_CERTIFICATE + WINDOWS_CERTIFICATE_PASSWORD + WINDOWS_CERTIFICATE_THUMBPRINT, Azure Trusted Signing variables, or WINDOWS_SIGN_COMMAND",
    );
  }
}

function requireAll(label, keys) {
  for (const key of keys) {
    if (!has(key)) {
      missing.push(`${label} requires ${key}`);
    }
  }
}

function has(key) {
  return typeof process.env[key] === "string" && process.env[key].trim().length > 0;
}

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
