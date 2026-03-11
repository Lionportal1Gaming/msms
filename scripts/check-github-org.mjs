import { execFileSync } from "node:child_process";
import {
  expectedRepoSlug,
  parseGithubArgs,
  REQUIRED_REPO_SECRETS,
  REQUIRED_REPO_VARIABLES,
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

function runJson(command, args) {
  const raw = run(command, args);
  return raw ? JSON.parse(raw) : null;
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

const githubRepo = runJson("gh", [
  "repo",
  "view",
  expectedRepoSlug(org, repo),
  "--json",
  "owner,name,url,isPrivate,defaultBranchRef"
]);

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
    `Expected ${expectedRepoSlug(org, repo)} default branch to be main. Received ${githubRepo.defaultBranchRef?.name || "unset"}.`
  );
}

const actionsPermissions = runJson("gh", [
  "api",
  `repos/${expectedRepoSlug(org, repo)}/actions/permissions`
]);

if (!actionsPermissions?.enabled) {
  fail(
    `GitHub Actions must be enabled for ${expectedRepoSlug(org, repo)} before the stable release dry run.`
  );
}

const repoVariables = runJson("gh", [
  "variable",
  "list",
  "--repo",
  expectedRepoSlug(org, repo),
  "--json",
  "name"
]);
const missingVariables = REQUIRED_REPO_VARIABLES.filter(
  (name) => !repoVariables?.some((variable) => variable.name === name)
);

if (missingVariables.length > 0) {
  fail(
    `Missing required GitHub repository variables in ${expectedRepoSlug(org, repo)}: ${missingVariables.join(", ")}.`
  );
}

const repoSecrets = runJson("gh", [
  "secret",
  "list",
  "--repo",
  expectedRepoSlug(org, repo),
  "--json",
  "name"
]);
const missingSecrets = REQUIRED_REPO_SECRETS.filter(
  (name) => !repoSecrets?.some((secret) => secret.name === name)
);

if (missingSecrets.length > 0) {
  fail(
    `Missing required GitHub repository secrets in ${expectedRepoSlug(org, repo)}: ${missingSecrets.join(", ")}.`
  );
}

const branchProtectionQuery = `
query($owner: String!, $repo: String!) {
  repository(owner: $owner, name: $repo) {
    branchProtectionRules(first: 20) {
      nodes {
        pattern
        requiresStatusChecks
        requiredStatusCheckContexts
      }
    }
  }
}
`;
const branchProtection = runJson("gh", [
  "api",
  "graphql",
  "-f",
  `query=${branchProtectionQuery}`,
  "-F",
  `owner=${org}`,
  "-F",
  `repo=${repo}`
]);
const mainProtectionRule =
  branchProtection?.data?.repository?.branchProtectionRules?.nodes?.find(
    (rule) => rule.pattern === "main"
  ) ?? null;

if (!mainProtectionRule) {
  fail(
    `Expected a branch protection rule for main in ${expectedRepoSlug(org, repo)} before the first stable release.`
  );
}

if (
  !mainProtectionRule.requiresStatusChecks ||
  !Array.isArray(mainProtectionRule.requiredStatusCheckContexts) ||
  mainProtectionRule.requiredStatusCheckContexts.length === 0
) {
  fail(
    `The main branch protection rule in ${expectedRepoSlug(org, repo)} must require status checks before stable release tags are used.`
  );
}

console.log(`Verified GitHub org repository ${expectedRepoSlug(org, repo)}: ${githubRepo.url}`);
