import { execFileSync } from "node:child_process";
import { parseGithubArgs } from "./github-org-config.mjs";

function runNodeScript(script, args) {
  execFileSync("node", [script, ...args], {
    stdio: "inherit"
  });
}

const forwardedArgs = process.argv.slice(2);
const { org, repo } = parseGithubArgs(forwardedArgs);

runNodeScript("scripts/validate-release.mjs", [
  "--require-gh",
  "--require-remote",
  ...forwardedArgs
]);

runNodeScript("scripts/check-github-org.mjs", ["--org", org, "--repo", repo]);
console.log("Release preflight passed.");
