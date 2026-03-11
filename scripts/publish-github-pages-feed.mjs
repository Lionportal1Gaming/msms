import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";

const args = process.argv.slice(2);

function getArg(name) {
  const index = args.indexOf(name);
  return index >= 0 ? args[index + 1] ?? null : null;
}

function requireArg(name) {
  const value = getArg(name);
  if (!value) {
    throw new Error(`Missing required argument ${name}.`);
  }
  return value;
}

function runInherited(command, commandArgs, options = {}) {
  execFileSync(command, commandArgs, {
    stdio: "inherit",
    ...options
  });
}

const channel = requireArg("--channel");
const assetPath = requireArg("--asset-path");
const repoSlug = process.env.GITHUB_REPOSITORY;
const githubToken = process.env.GITHUB_TOKEN;
const remoteUrl =
  process.env.GITHUB_PAGES_REMOTE_URL ??
  (repoSlug && githubToken
    ? `https://x-access-token:${githubToken}@github.com/${repoSlug}.git`
    : null);
const branch = "gh-pages";

if (!["stable", "beta"].includes(channel)) {
  throw new Error("Usage: node scripts/publish-github-pages-feed.mjs --channel <stable|beta> --asset-path <path>");
}

if (!remoteUrl) {
  throw new Error(
    "GITHUB_REPOSITORY and GITHUB_TOKEN are required unless GITHUB_PAGES_REMOTE_URL is provided."
  );
}

if (!fs.existsSync(assetPath)) {
  throw new Error(`Asset path does not exist: ${assetPath}`);
}

const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "msms-pages-feed-"));
const worktree = path.join(tempRoot, "pages");
let branchExists = false;
try {
  execFileSync(
    "bash",
    [
      "-lc",
      `git ls-remote --exit-code --heads "${remoteUrl}" ${branch} >/dev/null 2>&1`
    ],
    { stdio: "ignore" }
  );
  branchExists = true;
} catch {
  branchExists = false;
}

fs.mkdirSync(worktree, { recursive: true });
runInherited("git", ["init"], { cwd: worktree });
runInherited("git", ["config", "user.name", "github-actions[bot]"], { cwd: worktree });
runInherited("git", ["config", "user.email", "41898282+github-actions[bot]@users.noreply.github.com"], {
  cwd: worktree
});
runInherited("git", ["remote", "add", "origin", remoteUrl], { cwd: worktree });

if (branchExists) {
  runInherited("git", ["fetch", "--depth", "1", "origin", branch], { cwd: worktree });
  runInherited("git", ["checkout", "-B", branch, "FETCH_HEAD"], { cwd: worktree });
} else {
  runInherited("git", ["checkout", "--orphan", branch], { cwd: worktree });
  for (const entry of fs.readdirSync(worktree)) {
    if (entry === ".git") {
      continue;
    }
    fs.rmSync(path.join(worktree, entry), { recursive: true, force: true });
  }
}

const updatesDirectory = path.join(worktree, "updates", channel);
fs.mkdirSync(updatesDirectory, { recursive: true });
fs.copyFileSync(assetPath, path.join(updatesDirectory, "latest.json"));
fs.writeFileSync(path.join(worktree, ".nojekyll"), "");

runInherited("git", ["add", "."], { cwd: worktree });

let hasChanges = true;
try {
  runInherited("git", ["diff", "--cached", "--quiet"], { cwd: worktree });
  hasChanges = false;
} catch {
  hasChanges = true;
}

if (!hasChanges) {
  console.log(`GitHub Pages feed for ${channel} is already up to date.`);
  process.exit(0);
}

runInherited("git", ["commit", "-m", `chore(release): publish ${channel} updater feed`], {
  cwd: worktree
});
runInherited("git", ["push", "origin", `${branch}:${branch}`], { cwd: worktree });

console.log(`Published GitHub Pages updater feed for ${channel} to ${repoSlug ?? remoteUrl}.`);
