const { invoke } = window.__TAURI__.core;
const appWindow = window.__TAURI__.window.getCurrentWindow();

const $ = (id) => document.getElementById(id);

const state = { kind: "Map", folder: "", folders: [], created: "", lastKey: "", resultTitle: "" };

$("win-min").addEventListener("click", () => appWindow.minimize());
$("win-close").addEventListener("click", () => appWindow.close());
document.addEventListener("contextmenu", (e) => {
  if (!e.target.closest("input")) e.preventDefault();
});

const navButtons = [...document.querySelectorAll(".nv")];
function moveNavBar(btn) {
  const top = btn.offsetTop + (btn.offsetHeight - 18) / 2;
  $("nav-bar").style.transform = `translateY(${top}px)`;
}
navButtons.forEach((btn) =>
  btn.addEventListener("click", () => {
    if (btn.classList.contains("active")) return;
    closePop();
    navButtons.forEach((b) => b.classList.toggle("active", b === btn));
    document.querySelectorAll(".page").forEach((p) => p.classList.toggle("active", p.id === `page-${btn.dataset.page}`));
    moveNavBar(btn);
  }),
);

const kindButtons = [...document.querySelectorAll("#kind button")];
kindButtons.forEach((btn, i) =>
  btn.addEventListener("click", () => {
    state.kind = btn.dataset.kind;
    kindButtons.forEach((b) => b.classList.toggle("on", b === btn));
    $("kind-thumb").style.transform = `translateX(${i * 100}%)`;
  }),
);

function shortPath(path, max = 58) {
  if (path.length <= max) return path;
  const parts = path.split("\\");
  let tail = parts.pop();
  while (parts.length > 1 && (parts[0] + "\\…\\" + parts[parts.length - 1] + "\\" + tail).length <= max) {
    tail = parts.pop() + "\\" + tail;
  }
  const out = parts[0] + "\\…\\" + tail;
  return out.length <= max ? out : "…" + path.slice(-(max - 1));
}

function cleanName() {
  return $("name").value.trim().toLowerCase();
}

function fitMiddle(text, max) {
  if (text.length <= max) return text;
  const head = Math.ceil((max - 1) / 2);
  return text.slice(0, head) + "…" + text.slice(text.length - (max - 1 - head));
}

function renderTarget() {
  const name = cleanName();
  const target = $("target");
  if (state.folder && name) {
    const folder = state.folder.replace(/\\$/, "");
    const base = folder.length <= 24 ? folder : "…\\" + fitMiddle(folder.split("\\").pop(), 18);
    target.textContent = `${base}\\${fitMiddle(name, 26)}\\dlc.rpf`;
    target.title = `${folder}\\${name}\\dlc.rpf`;
  } else {
    target.textContent = "";
    target.title = "";
  }
  document.querySelectorAll(".pk").forEach((el) => (el.textContent = name || "mymap"));
}

const samePath = (a, b) => a.replace(/[\\/]+$/, "").toLowerCase() === b.replace(/[\\/]+$/, "").toLowerCase();

function renderFolder() {
  const el = $("folder");
  el.classList.toggle("empty", !state.folder);
  el.textContent = state.folder ? shortPath(state.folder) : "No folder selected";
  el.title = state.folder;
  renderTarget();
}

function renderPop() {
  const list = $("pop-list");
  list.replaceChildren();
  for (const item of state.folders) {
    const row = document.createElement("div");
    row.className = "pop-item";
    row.classList.toggle("on", !!state.folder && samePath(item.path, state.folder));
    row.classList.toggle("gone", !item.exists);

    const pick = document.createElement("button");
    pick.className = "pop-pick";
    pick.setAttribute("role", "option");
    pick.title = item.exists ? item.path : `${item.path} (missing)`;
    const icon = document.createElement("i");
    icon.className = item.exists ? "ti ti-folder" : "ti ti-alert-triangle";
    const label = document.createElement("span");
    label.textContent = shortPath(item.path, 54);
    const tick = document.createElement("i");
    tick.className = "ti ti-check tick";
    pick.append(icon, label, tick);
    pick.addEventListener("click", () => chooseFolder(item));

    const del = document.createElement("button");
    del.className = "pop-del";
    del.setAttribute("aria-label", "Remove from list");
    del.title = "Remove from list";
    del.innerHTML = '<i class="ti ti-x"></i>';
    del.addEventListener("click", () => removeFolder(item.path));

    row.append(pick, del);
    list.append(row);
  }
}

function applyFolders(f) {
  state.folders = f.items;
  state.folder = f.current;
  renderFolder();
  renderPop();
  if ($("pop").classList.contains("open")) placePop();
}

function placePop() {
  const page = $("page-create").getBoundingClientRect();
  const btn = $("folder-btn").getBoundingClientRect();
  const pop = $("pop");
  pop.style.left = `${btn.left - page.left}px`;
  pop.style.width = `${btn.width}px`;
  const limit = page.height - 8;
  const h = pop.offsetHeight;
  let top = btn.bottom - page.top + 6;
  if (top + h > limit) {
    const above = btn.top - page.top - 6 - h;
    top = above >= 12 ? above : Math.max(12, limit - h);
  }
  pop.style.top = `${top}px`;
}

function openPop() {
  if (!state.folders.length) {
    addFolder();
    return;
  }
  renderPop();
  placePop();
  $("pop").classList.add("open");
  $("folder-btn").classList.add("open");
  $("folder-btn").setAttribute("aria-expanded", "true");
  const current = $("pop").querySelector(".pop-item.on .pop-pick") || $("pop").querySelector(".pop-pick, .pop-add");
  current?.focus({ preventScroll: true });
}

function closePop(refocus = false) {
  $("pop").classList.remove("open");
  $("folder-btn").classList.remove("open");
  $("folder-btn").setAttribute("aria-expanded", "false");
  if (refocus) $("folder-btn").focus();
}

async function chooseFolder(item) {
  if (!item.exists) {
    closePop();
    showMessage("That folder no longer exists. Remove it from the list or add it again.");
    return;
  }
  try {
    applyFolders(await invoke("select_folder", { path: item.path }));
    closePop();
    showMessage("");
  } catch (e) {
    closePop();
    showMessage(String(e));
  }
}

async function removeFolder(path) {
  try {
    applyFolders(await invoke("remove_folder", { path }));
    if (!state.folders.length) closePop(true);
    else $("pop").querySelector(".pop-pick, .pop-add")?.focus({ preventScroll: true });
  } catch (e) {
    closePop();
    showMessage(String(e));
  }
}

async function addFolder() {
  closePop();
  try {
    const f = await invoke("pick_folder", { current: state.folder });
    if (f) {
      applyFolders(f);
      showMessage("");
    }
  } catch (e) {
    showMessage(String(e));
  }
}

$("folder-btn").addEventListener("click", () => ($("pop").classList.contains("open") ? closePop() : openPop()));
$("add").addEventListener("click", addFolder);
document.addEventListener("mousedown", (e) => {
  if (!$("pop").classList.contains("open")) return;
  if (!e.target.closest("#pop") && !e.target.closest("#folder-btn")) closePop();
});
document.addEventListener("keydown", (e) => {
  if (!$("pop").classList.contains("open")) return;
  if (e.key === "Escape") {
    closePop(true);
    return;
  }
  if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
  const items = [...$("pop").querySelectorAll(".pop-pick, .pop-add")];
  const i = items.indexOf(document.activeElement.closest(".pop-item")?.querySelector(".pop-pick") || document.activeElement);
  const next = e.key === "ArrowDown" ? (i + 1) % items.length : (i - 1 + items.length) % items.length;
  items[next].focus({ preventScroll: true });
  e.preventDefault();
});

function hideResult() {
  $("result").classList.remove("show");
}

function showMessage(text) {
  const el = $("message");
  el.textContent = text;
  el.title = text;
  if (text) hideResult();
  el.classList.toggle("show", !!text);
}

let noticeTimer;
function resultNotice(text) {
  clearTimeout(noticeTimer);
  const title = $("result-title");
  title.textContent = text;
  title.title = text;
  title.classList.add("bad");
  noticeTimer = setTimeout(() => {
    title.textContent = state.resultTitle;
    title.title = "";
    title.classList.remove("bad");
  }, 2600);
}

$("name").addEventListener("input", () => {
  $("name").classList.remove("bad");
  showMessage("");
  renderTarget();
});
$("name").addEventListener("keydown", (e) => {
  if (e.key === "Enter") create();
});

async function create() {
  const btn = $("create");
  if (btn.classList.contains("busy")) return;
  if (!state.folder) {
    showMessage("Choose where to save the pack first.");
    return;
  }
  const name = cleanName();
  if (!name || /\s/.test(name)) {
    $("name").classList.add("bad");
    $("name").focus();
    showMessage("Please enter a DLC name without spaces.");
    return;
  }
  const key = `${state.kind}|${state.folder}|${name}`;
  if (key === state.lastKey && $("result").classList.contains("show")) return;
  btn.classList.add("busy");
  const icon = btn.querySelector("i");
  icon.className = "ti ti-loader-2";
  showMessage("");
  hideResult();
  try {
    const res = await invoke("create_pack", { kind: state.kind, name, output: state.folder });
    state.created = res.rpf;
    state.lastKey = key;
    const used = state.folders.find((f) => samePath(f.path, state.folder)) || { path: state.folder, exists: true };
    state.folders = [used, ...state.folders.filter((f) => f !== used)];
    clearTimeout(noticeTimer);
    state.resultTitle = `${state.kind} pack created`;
    $("result-title").textContent = state.resultTitle;
    $("result-title").classList.remove("bad");
    $("item").textContent = res.item;
    $("item").title = res.item;
    $("copy").classList.remove("done");
    $("copy").querySelector("i").className = "ti ti-copy";
    requestAnimationFrame(() => $("result").classList.add("show"));
  } catch (e) {
    showMessage(String(e));
  } finally {
    btn.classList.remove("busy");
    icon.className = "ti ti-arrow-right";
  }
}
$("create").addEventListener("click", create);

$("copy").addEventListener("click", async () => {
  try {
    await invoke("copy_text", { text: $("item").textContent });
    $("copy").classList.add("done");
    $("copy").querySelector("i").className = "ti ti-check";
    setTimeout(() => {
      $("copy").classList.remove("done");
      $("copy").querySelector("i").className = "ti ti-copy";
    }, 1600);
  } catch (e) {
    resultNotice("Could not copy to clipboard.");
  }
});

$("open").addEventListener("click", async () => {
  try {
    await invoke("reveal", { path: state.created });
  } catch (e) {
    resultNotice(String(e));
  }
});

(async () => {
  moveNavBar(navButtons[0]);
  try {
    const f = await invoke("start");
    applyFolders(f);
    if (f.lost) showMessage(`The last folder no longer exists: ${shortPath(f.lost, 44)}`);
  } catch (e) {
    renderFolder();
    showMessage(String(e));
  }
  $("name").focus();
})();
