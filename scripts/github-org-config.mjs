import fs from "node:fs";
import path from "node:path";

const root = process.cwd();

export const DEFAULT_GITHUB_ORG = "Lionportal1Gaming";

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
