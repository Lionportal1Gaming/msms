import { execFileSync } from "node:child_process";

const tag = process.argv[2];

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
    "--json",
    "tagName,name,url,isDraft,isPrerelease"
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

console.log(`Verified GitHub release ${release.tagName}: ${release.url}`);
