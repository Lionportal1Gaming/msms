import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const args = process.argv.slice(2);
const channelIndex = args.indexOf("--channel");
const writeInPlace = args.includes("--write");
const channel = channelIndex >= 0 ? args[channelIndex + 1] : null;

if (!channel || !["stable", "beta"].includes(channel)) {
  throw new Error("Usage: node scripts/prepare-tauri-config.mjs --channel <stable|beta> [--write]");
}

const packageJson = JSON.parse(
  fs.readFileSync(path.join(root, "package.json"), "utf8")
);
const tauriConfigPath = path.join(root, "src-tauri", "tauri.conf.json");
const tauriConfig = JSON.parse(fs.readFileSync(tauriConfigPath, "utf8"));

const envKey =
  channel === "beta" ? "MSMS_UPDATER_BETA_ENDPOINT" : "MSMS_UPDATER_STABLE_ENDPOINT";
const endpoint = process.env[envKey];
const pubkey = process.env.MSMS_UPDATER_PUBLIC_KEY;

if (!endpoint || !pubkey) {
  throw new Error(
    `Missing updater configuration. Expected MSMS_UPDATER_PUBLIC_KEY and ${envKey}.`
  );
}

tauriConfig.version = packageJson.version;
tauriConfig.plugins ??= {};
tauriConfig.plugins.updater ??= {};
tauriConfig.plugins.updater.pubkey = pubkey;
tauriConfig.plugins.updater.endpoints = [endpoint];

const output = `${JSON.stringify(tauriConfig, null, 2)}\n`;
if (writeInPlace) {
  fs.writeFileSync(tauriConfigPath, output);
  console.log(`Prepared src-tauri/tauri.conf.json for the ${channel} channel.`);
} else {
  process.stdout.write(output);
}
