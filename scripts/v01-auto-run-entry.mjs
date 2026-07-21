#!/usr/bin/env node
/**
 * Cross-platform entry for `pnpm run verify:local`.
 * Dispatches to scripts/v01-auto-run.ps1 (Windows) or scripts/v01-auto-run.sh (POSIX).
 */
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const rawArgs = process.argv.slice(2);
const isWin = process.platform === "win32";

function mapWinArgs(args) {
  const mapped = [];
  for (const a of args) {
    if (a === "--skip-build") mapped.push("-SkipBuild");
    else if (a === "--strict-entry") mapped.push("-StrictEntry");
    else mapped.push(a);
  }
  return mapped;
}

let result;
if (isWin) {
  const script = path.join(root, "scripts", "v01-auto-run.ps1");
  const mapped = mapWinArgs(rawArgs);
  result = spawnSync("pwsh", ["-NoProfile", "-File", script, ...mapped], {
    stdio: "inherit",
    cwd: root,
  });
  if (result.error && result.error.code === "ENOENT") {
    result = spawnSync(
      "powershell.exe",
      ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", script, ...mapped],
      { stdio: "inherit", cwd: root },
    );
  }
} else {
  result = spawnSync("bash", [path.join(root, "scripts", "v01-auto-run.sh"), ...rawArgs], {
    stdio: "inherit",
    cwd: root,
  });
}

process.exit(result.status ?? 1);
