import { spawn } from "node:child_process";

const mode = process.argv[2] === "build" ? "build" : "dev";
const command = process.platform === "win32" ? "powershell.exe" : "pnpm";
const args = process.platform === "win32"
  ? ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "scripts/dev-windows.ps1", mode]
  : ["--filter", "@forge-wisper/desktop", "tauri", mode];

const child = spawn(command, args, { stdio: "inherit" });
child.on("error", (error) => {
  console.error(`Failed to start ${command}: ${error.message}`);
  process.exitCode = 1;
});
child.on("exit", (code, signal) => {
  process.exitCode = code ?? (signal ? 1 : 0);
});
