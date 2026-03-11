import { execFileSync } from "node:child_process";
import { expectedRepoSlug, parseGithubArgs } from "./github-org-config.mjs";

const values = process.argv.slice(2);
const { org, repo } = parseGithubArgs(values);
let tag = null;

for (let index = 0; index < values.length; index += 1) {
  const value = values[index];
  if (value === "--org" || value === "--repo") {
    index += 1;
    continue;
  }
  if (!value.startsWith("--")) {
    tag = value;
    break;
  }
}

if (!tag) {
  throw new Error("Usage: node scripts/verify-github-release.mjs <tag>");
}

try {
  execFileSync("gh", ["auth", "status"], { stdio: "ignore" });
} catch {
  throw new Error("GitHub CLI is not authenticated. Run `gh auth login` and retry.");
}

const raw = execFileSync(
  "gh",
  [
    "release",
    "view",
    tag,
    "--repo",
    expectedRepoSlug(org, repo),
    "--json",
    "tagName,name,url,isDraft,isPrerelease,assets"
  ],
  { encoding: "utf8" }
);
const release = JSON.parse(raw);

if (release.tagName !== tag) {
  throw new Error(`Expected release tag ${tag}, received ${release.tagName}.`);
}

if (tag.includes("-beta.") && !release.isPrerelease) {
  throw new Error(`Expected ${tag} to be marked as a prerelease on GitHub.`);
}

if (!tag.includes("-beta.") && release.isPrerelease) {
  throw new Error(`Expected ${tag} to be a stable release on GitHub.`);
}

if (release.isDraft) {
  throw new Error(`Expected ${tag} to be published, but it is still a draft.`);
}

if (!Array.isArray(release.assets) || release.assets.length === 0) {
  throw new Error(`Expected ${tag} to include release assets in ${expectedRepoSlug(org, repo)}.`);
}

if (!release.assets.some((asset) => asset.name?.endsWith(".json"))) {
  throw new Error(
    `Expected ${tag} to include updater metadata (.json) assets in ${expectedRepoSlug(org, repo)}.`
  );
}

console.log(
  `Verified GitHub release ${release.tagName} in ${expectedRepoSlug(org, repo)}: ${release.url}`
);
