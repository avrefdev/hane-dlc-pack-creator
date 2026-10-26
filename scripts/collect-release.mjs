import { copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync } from "node:fs";

const { version } = JSON.parse(readFileSync("package.json", "utf8"));
const target = "src-tauri/target/release";
const exe = `${target}/Hane DLC Pack Creator.exe`;
mkdirSync("release", { recursive: true });
copyFileSync(exe, "release/Hane DLC Pack Creator.exe");
const setup = readdirSync(`${target}/bundle/nsis`).find((f) => f.endsWith("-setup.exe"));
if (setup) copyFileSync(`${target}/bundle/nsis/${setup}`, `release/Hane DLC Pack Creator ${version} Setup.exe`);
if (existsSync("../exe")) copyFileSync(exe, "../exe/Hane DLC Pack Creator.exe");
console.log("release ready");
