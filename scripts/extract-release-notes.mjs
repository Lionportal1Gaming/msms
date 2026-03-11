import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const version = process.argv[2];

if (!version) {
  throw new Error("Usage: node scripts/extract-release-notes.mjs <version>");
}

const changelog = fs.readFileSync(path.join(root, "CHANGELOG.md"), "utf8");
const escapedVersion = version.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
const sectionPattern = new RegExp(
  `## \\[${escapedVersion}\\][^\\n]*\\n([\\s\\S]*?)(?=\\n## \\[|$)`,
  "m"
);
const match = changelog.match(sectionPattern);

if (!match) {
  throw new Error(`Unable to locate release notes for ${version} in CHANGELOG.md`);
}

const notes = match[1].trim();
if (process.env.GITHUB_OUTPUT) {
  fs.appendFileSync(process.env.GITHUB_OUTPUT, `body<<EOF\n${notes}\nEOF\n`);
} else {
  process.stdout.write(notes);
}
