const { invoke } = window.__TAURI__.core;
const appWindow = window.__TAURI__.window.getCurrentWindow();

const $ = (id) => document.getElementById(id);

const state = { kind: "Map", gta: null, resource: null, created: "", lastKey: "", resultTitle: "" };

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
    navButtons.forEach((b) => b.classList.toggle("active", b === btn));
    if (btn.dataset.page === "linked") loadLinks();
    document.querySelectorAll(".page").forEach((p) => p.classList.toggle("active", p.id === `page-${btn.dataset.page}`));
    moveNavBar(btn);
  }),
);

const kindButtons = [...document.querySelectorAll("#kind button")];
kindButtons.forEach((btn, i) =>
  btn.addEventListener("click", () => {
    state.kind = btn.dataset.kind;
    $("resource-row").classList.toggle("off", state.kind !== "Map");
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

function fitMiddle(text, max) {
  if (text.length <= max) return text;
  const head = Math.ceil((max - 1) / 2);
  return text.slice(0, head) + "…" + text.slice(text.length - (max - 1 - head));
}

function cleanName() {
  return $("name").value.trim().toLowerCase();
}

function renderTarget() {
  const name = cleanName();
  const target = $("target");
  if (state.gta?.valid && name) {
    target.textContent = `…\\dlcpacks\\${fitMiddle(name, 34)}\\dlc.rpf`;
    target.title = `${state.gta.dlcpacks}\\${name}\\dlc.rpf`;
  } else {
    target.textContent = "";
    target.title = "";
  }
  document.querySelectorAll(".pk").forEach((el) => (el.textContent = name || "mymap"));
}

function setMessage(id, text) {
  const el = $(id);
  el.textContent = text;
  el.title = text;
  el.classList.toggle("show", !!text);
}

function hideResult() {
  $("result").classList.remove("show");
}

function showMessage(text) {
  if (text) hideResult();
  setMessage("message", text);
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

function applyGta(gta) {
  state.gta = gta;
  $("gta-path").textContent = gta.path ? shortPath(gta.path, 60) : "Not set";
  $("gta-path").title = gta.path;
  $("gta-path").classList.toggle("bad", !gta.valid);
  $("dlcpacks-path").textContent = gta.valid ? shortPath(gta.dlcpacks, 60) : "";
  $("dlcpacks-path").title = gta.dlcpacks;
  renderTarget();
  const setup = $("setup");
  if (gta.valid) {
    setup.classList.remove("show");
    return;
  }
  $("setup-title").textContent = gta.path ? "GTA V was not found" : "Locate GTA V";
  $("setup-text").textContent = gta.path
    ? `GTA5.exe is no longer in ${shortPath(gta.path, 44)}. Select it again.`
    : "Select GTA5.exe from your GTA V folder.";
  setup.classList.add("show");
}

async function pickGta(messageId) {
  setMessage(messageId, "");
  try {
    const gta = await invoke("pick_gta");
    if (gta) {
      applyGta(gta);
      showMessage("");
      hideResult();
      state.lastKey = "";
    }
  } catch (e) {
    setMessage(messageId, String(e));
  }
}

$("setup-pick").addEventListener("click", () => pickGta("setup-message"));
$("gta-change").addEventListener("click", () => pickGta("settings-message"));
$("dlcpacks-open").addEventListener("click", async () => {
  try {
    await invoke("reveal", { path: state.gta?.dlcpacks || "" });
    setMessage("settings-message", "");
  } catch (e) {
    setMessage("settings-message", String(e));
  }
});

function renderResource() {
  const r = state.resource;
  const path = $("resource-path");
  path.textContent = r ? shortPath(r.path, 46) : "Not linked";
  path.title = r ? r.path : "";
  path.classList.toggle("empty", !r);
  $("resource-count").textContent = r ? `${r.meta + r.main} files` : "";
  $("resource-count").title = r ? `${r.meta} to metadata, ${r.main} to the main rpf` : "";
  $("resource-clear").hidden = !r;
  $("resource-pick").textContent = r ? "Change" : "Select";
}

$("resource-pick").addEventListener("click", async () => {
  try {
    const r = await invoke("pick_resource");
    if (r) {
      state.resource = r;
      renderResource();
      showMessage("");
    }
  } catch (e) {
    showMessage(String(e));
  }
});
$("resource-clear").addEventListener("click", () => {
  state.resource = null;
  renderResource();
});

function linkRow(link) {
  const row = document.createElement("div");
  row.className = "link";
  const name = document.createElement("span");
  name.className = "link-name";
  name.textContent = link.name;
  name.title = link.name;
  const path = document.createElement("span");
  path.className = "link-path";
  path.classList.toggle("bad", !link.resourceExists);
  path.textContent = link.resourceExists ? shortPath(link.resource, 44) : `Missing: ${shortPath(link.resource, 34)}`;
  path.title = link.resource;
  const time = document.createElement("span");
  time.className = "link-time";
  time.textContent = link.synced ? link.synced.slice(0, 16) : "";
  time.title = link.synced ? `Last sync ${link.synced}` : "";
  const sync = document.createElement("button");
  sync.className = "ghost";
  sync.innerHTML = '<i class="ti ti-refresh"></i>Sync';
  sync.addEventListener("click", () => syncLink(link.name, sync, time));
  const un = document.createElement("button");
  un.className = "icon";
  un.title = "Unlink (the pack is kept)";
  un.setAttribute("aria-label", "Unlink");
  un.innerHTML = '<i class="ti ti-x"></i>';
  un.addEventListener("click", () => unlink(link.name));
  row.append(name, path, time, sync, un);
  return row;
}

function renderLinks(list) {
  const box = $("links");
  box.replaceChildren(...list.map((l, i) => {
    const row = linkRow(l);
    row.style.animationDelay = `${(i + 1) * 45}ms`;
    return row;
  }));
  $("links-empty").hidden = list.length > 0;
}

async function loadLinks() {
  setMessage("links-message", "");
  try {
    renderLinks(await invoke("links"));
  } catch (e) {
    setMessage("links-message", String(e));
  }
}

async function syncLink(name, btn, time) {
  btn.classList.add("busy");
  setMessage("links-message", "");
  time.className = "link-time";
  time.textContent = "Syncing…";
  try {
    const list = await invoke("sync_link", { name });
    renderLinks(list);
    const fresh = [...$("links").children].find((r) => r.querySelector(".link-name").textContent === name);
    if (fresh) {
      const t = fresh.querySelector(".link-time");
      t.textContent = "Synced just now";
      t.classList.add("ok");
    }
  } catch (e) {
    btn.classList.remove("busy");
    time.className = "link-time err";
    time.textContent = "Sync failed";
    time.title = String(e);
    setMessage("links-message", `${name}: ${e}`);
  }
}

async function unlink(name) {
  try {
    renderLinks(await invoke("unlink", { name }));
  } catch (e) {
    setMessage("links-message", String(e));
  }
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
  const name = cleanName();
  if (!name || /\s/.test(name)) {
    $("name").classList.add("bad");
    $("name").focus();
    showMessage("Please enter a DLC name without spaces.");
    return;
  }
  const resource = state.kind === "Map" && state.resource ? state.resource.path : null;
  const key = `${state.kind}|${state.gta?.path}|${name}|${resource}`;
  if (key === state.lastKey && $("result").classList.contains("show")) return;
  btn.classList.add("busy");
  const icon = btn.querySelector("i");
  icon.className = "ti ti-loader-2";
  showMessage("");
  hideResult();
  try {
    const res = await invoke("create_pack", { kind: state.kind, name, resource });
    state.created = res.rpf;
    state.lastKey = key;
    clearTimeout(noticeTimer);
    const files = res.meta + res.main;
    state.resultTitle = files ? `Map pack created with ${files} files, linked` : `${state.kind} pack created`;
    if (resource) {
      state.resource = null;
      renderResource();
    }
    $("result-title").textContent = state.resultTitle;
    $("result-title").classList.remove("bad");
    $("item").textContent = res.item;
    $("item").title = res.item;
    $("copy").classList.remove("done");
    $("copy").querySelector("i").className = "ti ti-copy";
    requestAnimationFrame(() => $("result").classList.add("show"));
  } catch (e) {
    showMessage(String(e));
    try {
      const gta = await invoke("start");
      if (!gta.valid) applyGta(gta);
    } catch (err) {
      showMessage(String(err));
    }
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
  renderResource();
  try {
    applyGta(await invoke("start"));
  } catch (e) {
    applyGta({ path: "", valid: false, dlcpacks: "" });
    setMessage("setup-message", String(e));
  }
  if (state.gta?.valid) $("name").focus();
})();
