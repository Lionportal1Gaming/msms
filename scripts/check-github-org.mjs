import { execFileSync } from "node:child_process";
import {
  expectedRepoSlug,
  parseGithubArgs,
  remoteMatchesOrgRepo
} from "./github-org-config.mjs";

const { org, repo } = parseGithubArgs(process.argv.slice(2));

function fail(message) {
  throw new Error(message);
}

function run(command, args) {
  return execFileSync(command, args, {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"]
  }).trim();
}

function ensureGhAuth() {
  try {
    execFileSync("gh", ["auth", "status"], { stdio: "ignore" });
  } catch {
    fail("GitHub CLI is not authenticated. Run `gh auth login -h github.com` and retry.");
  }
}

let remoteUrl = "";
try {
  remoteUrl = run("git", ["remote", "get-url", "origin"]);
} catch {
  fail(
    `Git remote \`origin\` is not configured. Bootstrap ${expectedRepoSlug(org, repo)} before running release preflight.`
  );
}

if (!remoteMatchesOrgRepo(remoteUrl, org, repo)) {
  fail(
    `Git remote origin must point to github.com/${org}/${repo}. Current origin is ${remoteUrl}.`
  );
}

ensureGhAuth();

const rawRepo = run("gh", [
  "repo",
  "view",
  expectedRepoSlug(org, repo),
  "--json",
  "owner,name,url,isPrivate,defaultBranchRef"
]);
const githubRepo = JSON.parse(rawRepo);

if (githubRepo.owner?.login !== org) {
  fail(
    `GitHub repository owner mismatch. Expected ${org}, received ${githubRepo.owner?.login ?? "unknown"}.`
  );
}

if (githubRepo.name !== repo) {
  fail(`GitHub repository name mismatch. Expected ${repo}, received ${githubRepo.name}.`);
}

if (!githubRepo.isPrivate) {
  fail(`Expected ${expectedRepoSlug(org, repo)} to be private for MVP release work.`);
}

if (githubRepo.defaultBranchRef?.name !== "main") {
  fail(
    `Expected ${expectedRepoSlug(org, repo)} default branch to be main. Received ${githubRepo.defaultBranchRef?.name ?? "unknown"}.`
  );
}

console.log(`Verified GitHub org repository ${expectedRepoSlug(org, repo)}: ${githubRepo.url}`);
