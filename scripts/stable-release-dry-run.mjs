import { execFileSync } from "node:child_process";
import { expectedRepoSlug, isStableTag, parseGithubArgs } from "./github-org-config.mjs";

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
  throw new Error("Usage: node scripts/stable-release-dry-run.mjs <stable-tag>");
}

if (!isStableTag(tag)) {
  throw new Error(`Stable release dry runs only accept vX.Y.Z tags. Received ${tag}.`);
}

function runNodeScript(script, args) {
  execFileSync("node", [script, ...args], {
    stdio: "inherit"
  });
}

runNodeScript("scripts/validate-release.mjs", [
  "--channel",
  "stable",
  "--tag",
  tag,
  "--require-gh",
  "--require-remote",
  "--org",
  org,
  "--repo",
  repo
]);

runNodeScript("scripts/check-github-org.mjs", ["--org", org, "--repo", repo]);

runNodeScript("scripts/verify-github-release.mjs", ["--org", org, "--repo", repo, tag]);

console.log(
  `Stable release dry run passed for ${tag} against ${expectedRepoSlug(org, repo)}.`
);
