import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import {
  expectedRepoSlug,
  parseGithubArgs,
  releaseVersionFromTag,
  remoteMatchesOrgRepo,
  REQUIRED_REPO_SECRETS,
  REQUIRED_REPO_VARIABLES
} from "./github-org-config.mjs";

const root = process.cwd();
const args = parseArgs(process.argv.slice(2));

function readJson(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

function readText(relativePath) {
  return fs.readFileSync(path.join(root, relativePath), "utf8");
}

function parseArgs(values) {
  const parsed = {
    channel: null,
    requireGh: false,
    requireRemote: false,
    requireSecrets: false,
    tag: null,
    ...parseGithubArgs(values)
  };

  for (let index = 0; index < values.length; index += 1) {
    const value = values[index];
    if (value === "--require-gh") {
      parsed.requireGh = true;
      continue;
    }
    if (value === "--require-remote") {
      parsed.requireRemote = true;
      continue;
    }
    if (value === "--require-secrets") {
      parsed.requireSecrets = true;
      continue;
    }
    if (value === "--channel") {
      parsed.channel = values[index + 1] ?? null;
      index += 1;
      continue;
    }
    if (value === "--tag") {
      parsed.tag = values[index + 1] ?? null;
      index += 1;
    }
  }

  return parsed;
}

function fail(message) {
  throw new Error(message);
}

function isMissingOrPlaceholder(value) {
  return (
    !value ||
    value.includes("REPLACE_WITH") ||
    value.includes("example.com") ||
    value.includes("__MSMS_")
  );
}

const packageJson = readJson("package.json");
const packageVersion = packageJson.version;
const changelog = readText("CHANGELOG.md");
const tauriConfig = readJson("src-tauri/tauri.conf.json");

if (!changelog.includes(`## [${packageVersion}]`)) {
  fail(`CHANGELOG.md is missing the ${packageVersion} release heading.`);
}

const cargoToml = readText("src-tauri/Cargo.toml");
const cargoMatch = cargoToml.match(/^version = "([^"]+)"/m);
if (!cargoMatch) {
  fail("Unable to locate Rust package version in src-tauri/Cargo.toml.");
}

if (cargoMatch[1] !== packageVersion) {
  fail(`Version mismatch: package.json=${packageVersion}, Cargo.toml=${cargoMatch[1]}`);
}

if (tauriConfig.version !== packageVersion) {
  fail(`Version mismatch: package.json=${packageVersion}, tauri.conf.json=${tauriConfig.version}`);
}

const updaterConfig = tauriConfig.plugins?.updater;
if (!updaterConfig?.active) {
  fail("Tauri updater plugin must remain active for release builds.");
}

if (!Array.isArray(updaterConfig.endpoints) || updaterConfig.endpoints.length !== 1) {
  fail("tauri.conf.json must define exactly one updater endpoint slot.");
}

if (updaterConfig.pubkey === "REPLACE_WITH_TAURI_UPDATER_PUBLIC_KEY") {
  fail("tauri.conf.json still uses the generic updater public key placeholder.");
}

if (updaterConfig.endpoints.some((endpoint) => endpoint.includes("example.com"))) {
  fail("tauri.conf.json still uses example.com updater endpoints.");
}

const envExample = readText(".env.example");
for (const key of [
  "MSMS_UPDATER_PUBLIC_KEY",
  "MSMS_UPDATER_STABLE_ENDPOINT",
  "MSMS_UPDATER_BETA_ENDPOINT"
]) {
  if (!envExample.includes(`${key}=`)) {
    fail(`.env.example is missing ${key}.`);
  }
}

if (args.tag) {
  if (!args.tag.startsWith("v")) {
    fail(`Release tags must start with v. Received ${args.tag}.`);
  }
  const tagVersion = releaseVersionFromTag(args.tag);
  if (!tagVersion) {
    fail(`Release tag ${args.tag} must use vX.Y.Z or vX.Y.Z-beta.N format.`);
  }
  if (tagVersion !== packageVersion) {
    fail(
      `Release tag ${args.tag} does not match package version ${packageVersion}.`
    );
  }
}

if (args.channel) {
  if (!["stable", "beta"].includes(args.channel)) {
    fail(`Unknown release channel ${args.channel}.`);
  }
  if (args.channel === "beta" && args.tag && !args.tag.includes("-beta.")) {
    fail(`Beta releases must use beta tags. Received ${args.tag}.`);
  }
  if (args.channel === "stable" && args.tag && args.tag.includes("-beta.")) {
    fail(`Stable releases must not use beta tags. Received ${args.tag}.`);
  }
}

if (args.requireGh) {
  try {
    execFileSync("gh", ["--version"], { stdio: "ignore" });
  } catch {
    fail("GitHub CLI is required for the local release workflow. Install gh and retry.");
  }

  try {
    execFileSync("gh", ["auth", "status"], { stdio: "ignore" });
  } catch {
    fail("GitHub CLI is installed but not authenticated. Run `gh auth login -h github.com` before release preflight.");
  }
}

if (args.requireRemote) {
  let remoteUrl = "";
  try {
    remoteUrl = execFileSync("git", ["remote", "get-url", "origin"], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"]
    }).trim();
  } catch {
    fail(
      `Git remote origin is missing. Bootstrap ${expectedRepoSlug(args.org, args.repo)} before release preflight.`
    );
  }

  if (!remoteMatchesOrgRepo(remoteUrl, args.org, args.repo)) {
    fail(
      `Git remote origin must point to github.com/${args.org}/${args.repo}. Current origin is ${remoteUrl}.`
    );
  }

  if (!args.requireGh) {
    try {
      execFileSync("gh", ["auth", "status"], { stdio: "ignore" });
    } catch {
      fail("GitHub CLI is installed but not authenticated. Run `gh auth login -h github.com` before release preflight.");
    }
  }

  try {
    const rawRepo = execFileSync(
      "gh",
      [
        "repo",
        "view",
        expectedRepoSlug(args.org, args.repo),
        "--json",
        "owner,name"
      ],
      {
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"]
      }
    );
    const repo = JSON.parse(rawRepo);
    if (repo.owner?.login !== args.org) {
      fail(
        `GitHub repo owner mismatch. Expected ${args.org}, received ${repo.owner?.login ?? "unknown"}.`
      );
    }
    if (repo.name !== args.repo) {
      fail(`GitHub repo name mismatch. Expected ${args.repo}, received ${repo.name}.`);
    }
  } catch (error) {
    if (error instanceof Error) {
      fail(error.message);
    }
    fail(`Unable to verify GitHub repository ${expectedRepoSlug(args.org, args.repo)}.`);
  }
}

if (args.requireSecrets) {
  const requiredEnv = [...REQUIRED_REPO_VARIABLES, ...REQUIRED_REPO_SECRETS];
  for (const name of requiredEnv) {
    if (isMissingOrPlaceholder(process.env[name] ?? "")) {
      fail(`Missing or placeholder release secret/environment value: ${name}.`);
    }
  }
}

console.log(`Validated release metadata for version ${packageVersion}.`);
