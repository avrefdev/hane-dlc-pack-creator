import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";

mkdirSync("ui/fonts", { recursive: true });
const inter = "node_modules/@fontsource-variable/inter/files";
for (const s of ["latin", "latin-ext"]) copyFileSync(`${inter}/inter-${s}-wght-normal.woff2`, `ui/fonts/inter-${s}.woff2`);

const tabler = "node_modules/@tabler/icons-webfont/dist";
copyFileSync(`${tabler}/fonts/tabler-icons.woff2`, "ui/fonts/tabler-icons.woff2");
const css = readFileSync(`${tabler}/tabler-icons.min.css`, "utf8");
const used = ["package", "book-2", "minus", "x", "map-2", "car", "folder", "folder-open", "copy", "check", "alert-triangle", "arrow-right", "loader-2", "chevron-down", "plus"];
const clean = css.replace(/\/\*[\s\S]*?\*\//g, "");
const head = clean.slice(0, clean.indexOf(".ti-"));
const rules = used.map((n) => {
  const m = css.match(new RegExp(`\\.ti-${n}:before\\{content:"([^"]+)"\\}`));
  if (!m) throw new Error(`icon not found: ${n}`);
  return `.ti-${n}:before{content:"${m[1]}"}`;
});
writeFileSync("ui/icons.css", head.replace(/src:[^;}]+/, 'src:url("fonts/tabler-icons.woff2") format("woff2")') + rules.join(""));
console.log("assets ready");
