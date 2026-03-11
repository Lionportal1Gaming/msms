import fs from "node:fs";
import path from "node:path";

const root = process.cwd();

export const DEFAULT_GITHUB_ORG = "Lionportal1Gaming";
export const REQUIRED_REPO_VARIABLES = [
  "MSMS_UPDATER_PUBLIC_KEY",
  "MSMS_UPDATER_STABLE_ENDPOINT",
  "MSMS_UPDATER_BETA_ENDPOINT"
];
export const REQUIRED_REPO_SECRETS = [
  "TAURI_SIGNING_PRIVATE_KEY",
  "TAURI_SIGNING_PRIVATE_KEY_PASSWORD"
];
export const REQUIRED_STABLE_RELEASE_ASSET_GROUPS = [
  {
    label: "Windows installer",
    patterns: [/\.msi$/i, /\.exe$/i]
  },
  {
    label: "macOS installer or updater bundle",
    patterns: [/\.dmg$/i, /\.app\.tar\.gz$/i]
  },
  {
    label: "Linux bundle",
    patterns: [/\.appimage$/i, /\.deb$/i, /\.rpm$/i]
  },
  {
    label: "stable updater metadata",
    patterns: [/^latest\.json$/i, /latest\.json$/i]
  }
];

export function readPackageJson() {
  return JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
}

export function defaultRepoName() {
  return readPackageJson().name;
}

export function parseGithubArgs(values) {
  const parsed = {
    org: DEFAULT_GITHUB_ORG,
    repo: defaultRepoName()
  };

  for (let index = 0; index < values.length; index += 1) {
    const value = values[index];
    if (value === "--org") {
      parsed.org = values[index + 1] ?? parsed.org;
      index += 1;
      continue;
    }
    if (value === "--repo") {
      parsed.repo = values[index + 1] ?? parsed.repo;
      index += 1;
    }
  }

  return parsed;
}

export function expectedRepoSlug(org, repo) {
  return `${org}/${repo}`;
}

export function normalizeRemoteUrl(remoteUrl) {
  return remoteUrl.trim().replace(/\.git$/, "");
}

export function remoteMatchesOrgRepo(remoteUrl, org, repo) {
  const normalized = normalizeRemoteUrl(remoteUrl);
  return (
    normalized === `git@github.com:${org}/${repo}` ||
    normalized === `https://github.com/${org}/${repo}` ||
    normalized === `ssh://git@github.com/${org}/${repo}`
  );
}

export function isStableTag(tag) {
  return /^v\d+\.\d+\.\d+$/.test(tag);
}

export function isBetaTag(tag) {
  return /^v\d+\.\d+\.\d+-beta\.\d+$/.test(tag);
}

export function releaseVersionFromTag(tag) {
  if (isStableTag(tag)) {
    return tag.slice(1);
  }

  const betaMatch = tag.match(/^v(\d+\.\d+\.\d+)-beta\.\d+$/);
  if (betaMatch) {
    return betaMatch[1];
  }

  return null;
}
