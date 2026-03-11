import { execFileSync } from "node:child_process";
import {
  expectedRepoSlug,
  parseGithubArgs,
  readPackageJson,
  remoteMatchesOrgRepo
} from "./github-org-config.mjs";

const { org, repo } = parseGithubArgs(process.argv.slice(2));
const packageJson = readPackageJson();

function fail(message) {
  throw new Error(message);
}

function run(command, args) {
  return execFileSync(command, args, {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"]
  }).trim();
}

function runInherited(command, args) {
  execFileSync(command, args, {
    stdio: "inherit"
  });
}

try {
  execFileSync("gh", ["auth", "status"], { stdio: "ignore" });
} catch {
  fail("GitHub CLI is not authenticated. Run `gh auth login -h github.com` before bootstrap.");
}

let originUrl = "";
try {
  originUrl = run("git", ["remote", "get-url", "origin"]);
} catch {
  originUrl = "";
}

if (originUrl) {
  if (!remoteMatchesOrgRepo(originUrl, org, repo)) {
    fail(
      `Existing origin points to ${originUrl}. Refusing to overwrite it because pushes must target ${expectedRepoSlug(org, repo)}.`
    );
  }
} else {
  runInherited("gh", [
    "repo",
    "create",
    expectedRepoSlug(org, repo),
    "--private",
    "--source=.",
    "--remote=origin",
    "--push=false",
    "--description",
    packageJson.description
  ]);
}

const repoDetails = JSON.parse(
  run("gh", [
    "repo",
    "view",
    expectedRepoSlug(org, repo),
    "--json",
    "owner,name,url,isPrivate,defaultBranchRef"
  ])
);

if (repoDetails.owner?.login !== org || repoDetails.name !== repo) {
  fail(`GitHub repository verification failed for ${expectedRepoSlug(org, repo)}.`);
}

if (!repoDetails.isPrivate) {
  fail(`Expected ${expectedRepoSlug(org, repo)} to be private.`);
}

console.log(`Bootstrap confirmed for ${expectedRepoSlug(org, repo)}: ${repoDetails.url}`);
console.log("Configure Actions variables and secrets before the first release tag:");
console.log("- MSMS_UPDATER_PUBLIC_KEY");
console.log("- MSMS_UPDATER_STABLE_ENDPOINT");
console.log("- MSMS_UPDATER_BETA_ENDPOINT");
console.log("- TAURI_SIGNING_PRIVATE_KEY");
console.log("- TAURI_SIGNING_PRIVATE_KEY_PASSWORD");
