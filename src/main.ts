import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { appWindow } from "@tauri-apps/api/window";

// ─── Default Config (matches bongocat-osu format) ─────────────────────────────
const DEFAULT_CONFIG: any = {
  mode: 1,
  resolution: { letterboxing: false, width: 1920, height: 1080, horizontalPosition: 0, verticalPosition: 0 },
  decoration: { leftHanded: false, rgb: [255, 255, 255], offsetX: [0, 11], offsetY: [0, -65], scalar: [1.0, 1.0] },
  osu: { mouse: true, toggleSmoke: false, paw: [255, 255, 255], pawEdge: [0, 0, 0], key1: [90], key2: [88], smoke: [67], wave: [] },
  taiko: { leftCentre: [88], rightCentre: [67], leftRim: [90], rightRim: [86] },
  catch: { left: [37], right: [39], dash: [16] },
  mania: { "4K": true, key4K: [68, 70, 74, 75], key7K: [83, 68, 70, 32, 74, 75, 76] },
  mousePaw: { pawStartingPoint: [211, 159], pawEndingPoint: [258, 228] }
};

// ─── Asset Paths ──────────────────────────────────────────────────────────────
const TRANSPARENT_1X1 = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=";

const imagePaths: Record<string, string> = {
  mousebg: 'img/osu/mousebg.png', tabletbg: 'img/osu/tabletbg.png',
  osu_left: 'img/osu/left.png', osu_right: 'img/osu/right.png', osu_up: 'img/osu/up.png',
  mouse: 'img/osu/mouse.png', tablet: 'img/osu/tablet.png',
  smoke: 'img/osu/smoke.png', wave: 'img/osu/wave.png',
  taiko_bg: 'img/taiko/bg.png',
  taiko_leftrim: 'img/taiko/leftrim.png', taiko_leftcentre: 'img/taiko/leftcentre.png', taiko_leftup: 'img/taiko/leftup.png',
  taiko_rightrim: 'img/taiko/rightrim.png', taiko_rightcentre: 'img/taiko/rightcentre.png', taiko_rightup: 'img/taiko/rightup.png',
  catch_bg: 'img/catch/bg.png', catch_left: 'img/catch/left.png', catch_right: 'img/catch/right.png',
  catch_up: 'img/catch/up.png', catch_dash: 'img/catch/dash.png', catch_mid: 'img/catch/mid.png',
  mania_bg_4K: 'img/mania/4K/bg.png', mania_bg_7K: 'img/mania/7K/bg.png',
  mania_leftup: 'img/mania/leftup.png', mania_left0: 'img/mania/left0.png',
  mania_left1: 'img/mania/left1.png', mania_left2: 'img/mania/left2.png',
  mania_rightup: 'img/mania/rightup.png', mania_right0: 'img/mania/right0.png',
  mania_right1: 'img/mania/right1.png', mania_right2: 'img/mania/right2.png',
  key_4K_0: 'img/mania/4K/0.png', key_4K_1: 'img/mania/4K/1.png',
  key_4K_2: 'img/mania/4K/2.png', key_4K_3: 'img/mania/4K/3.png',
  key_7K_0: 'img/mania/7K/0.png', key_7K_1: 'img/mania/7K/1.png',
  key_7K_2: 'img/mania/7K/2.png', key_7K_3: 'img/mania/7K/3.png',
  key_7K_4: 'img/mania/7K/4.png', key_7K_5: 'img/mania/7K/5.png', key_7K_6: 'img/mania/7K/6.png',
};

// ─── Global State ─────────────────────────────────────────────────────────────
let config: any = { ...DEFAULT_CONFIG };
const loadedImages = new Map<string, HTMLImageElement>();
const keysPressed = new Set<number>();
const mousePos = { x: 960, y: 540 }; // center of 1920x1080
let currentSkin = "default";
let globalInputActive = false;
let isLocked = false;

// Smoke particle system
interface SmokeParticle {
  x: number;
  y: number;
  alpha: number;
  size: number;
}
const smokeParticles: SmokeParticle[] = [];
let smokeToggled = false;
let smokeKeyWasDown = false;

// Key Binding State
let activeBindingButton: HTMLButtonElement | null = null;
let activeBindingPath: string | null = null;

let canvas: HTMLCanvasElement;
let ctx: CanvasRenderingContext2D;

// ─── Key Helper ───────────────────────────────────────────────────────────────
function getKeyName(keyCode: number): string {
  if (keyCode >= 65 && keyCode <= 90) return String.fromCharCode(keyCode);
  if (keyCode >= 48 && keyCode <= 57) return String.fromCharCode(keyCode);
  const special: Record<number, string> = {
    8: "Backspace", 9: "Tab", 13: "Enter", 16: "Shift", 17: "Ctrl", 18: "Alt",
    20: "CapsLock", 27: "Esc", 32: "Space", 33: "PageUp", 34: "PageDown",
    35: "End", 36: "Home", 37: "←", 38: "↑", 39: "→", 40: "↓",
    46: "Delete", 96: "Num0", 97: "Num1", 98: "Num2", 99: "Num3", 100: "Num4",
    101: "Num5", 102: "Num6", 103: "Num7", 104: "Num8", 105: "Num9",
    106: "Num*", 107: "Num+", 109: "Num-", 111: "Num/", 112: "F1", 113: "F2",
    114: "F3", 115: "F4", 116: "F5", 117: "F6", 118: "F7", 119: "F8", 120: "F9",
    121: "F10", 122: "F11", 123: "F12", 186: ";", 187: "=", 188: ",", 189: "-",
    190: ".", 191: "/", 192: "`", 219: "[", 220: "\\", 221: "]", 222: "'"
  };
  return special[keyCode] || `Code ${keyCode}`;
}

// ─── Config Getters/Setters ───────────────────────────────────────────────────
function getConfigVal(path: string): any {
  const parts = path.split('.');
  let obj = config;
  for (let i = 0; i < parts.length; i++) {
    if (obj === undefined || obj === null) return undefined;
    const key = parts[i];
    if (!isNaN(Number(key))) {
      obj = obj[Number(key)];
    } else {
      obj = obj[key];
    }
  }
  return obj;
}

function setConfigVal(path: string, val: any) {
  const parts = path.split('.');
  let obj = config;
  for (let i = 0; i < parts.length - 1; i++) {
    const key = parts[i];
    if (!obj[key]) {
      obj[key] = isNaN(Number(parts[i+1])) ? {} : [];
    }
    obj = obj[key];
  }
  const lastKey = parts[parts.length - 1];
  if (!isNaN(Number(lastKey))) {
    obj[Number(lastKey)] = val;
  } else {
    obj[lastKey] = val;
  }
}

// ─── Settings Controls ────────────────────────────────────────────────────────
function updateBindingButtons() {
  const buttons = document.querySelectorAll('.key-bind-btn');
  buttons.forEach(btn => {
    const path = btn.getAttribute('data-config-path');
    if (!path) return;
    const val = getConfigVal(path);
    if (Array.isArray(val)) {
      if (val.length === 0) {
        btn.textContent = "None";
      } else {
        btn.textContent = val.map(code => getKeyName(code)).join(" / ");
      }
    } else if (typeof val === 'number') {
      btn.textContent = getKeyName(val);
    } else {
      btn.textContent = "None";
    }
  });
}

function renderManiaKeysBindingUI() {
  const container = document.getElementById("mania-keys-container");
  if (!container) return;
  container.innerHTML = "";
  
  const is4K = config.mania?.['4K'] !== false;
  const count = is4K ? 4 : 7;
  const arrayName = is4K ? 'key4K' : 'key7K';
  
  if (!config.mania) config.mania = {};
  if (!config.mania[arrayName]) {
    config.mania[arrayName] = is4K ? [68, 70, 74, 75] : [83, 68, 70, 32, 74, 75, 76];
  }
  
  for (let i = 0; i < count; i++) {
    const row = document.createElement("div");
    row.className = "binding-row";
    
    const label = document.createElement("label");
    label.textContent = `Column ${i + 1}:`;
    
    const btn = document.createElement("button");
    btn.className = "key-bind-btn";
    btn.setAttribute("data-config-path", `mania.${arrayName}.${i}`);
    
    btn.addEventListener("click", () => startBinding(btn, `mania.${arrayName}.${i}`));
    
    row.appendChild(label);
    row.appendChild(btn);
    container.appendChild(row);
  }
  
  updateBindingButtons();
}

function startBinding(btn: HTMLButtonElement, path: string) {
  if (activeBindingButton) {
    activeBindingButton.classList.remove("recording");
    updateBindingButtons();
  }
  
  activeBindingButton = btn;
  activeBindingPath = path;
  btn.classList.add("recording");
  btn.textContent = "Press key...";
}

function handleKeyForBinding(keyCode: number): boolean {
  if (activeBindingButton && activeBindingPath) {
    if (keyCode === 27) { // Escape clears/cancels
      activeBindingButton.classList.remove("recording");
      activeBindingButton = null;
      activeBindingPath = null;
      updateBindingButtons();
      showToast("Binding cancelled");
      return true;
    }
    
    const isArrayField = activeBindingPath.includes("osu.") || 
                         activeBindingPath.includes("taiko.") || 
                         activeBindingPath.includes("catch.");
                         
    if (isArrayField) {
      setConfigVal(activeBindingPath, [keyCode]);
    } else {
      setConfigVal(activeBindingPath, keyCode);
    }
    
    activeBindingButton.classList.remove("recording");
    activeBindingButton = null;
    activeBindingPath = null;
    updateBindingButtons();
    showToast("✔ Key bound");
    return true;
  }
  return false;
}

// ─── Skin Loading ─────────────────────────────────────────────────────────────
async function loadSkin(skinName: string) {
  let configStr = "";
  try {
    configStr = await invoke<string>('read_skin_config', { skinName });
    config = JSON.parse(configStr);
  } catch (_e) {
    config = { ...DEFAULT_CONFIG };
  }

  // Update screen resolution on Rust side for relative evdev mouse accumulation
  const res = config.resolution || { width: 1920, height: 1080 };
  try {
    await invoke('update_screen_resolution', { width: res.width || 1920, height: res.height || 1080 });
  } catch (err) {
    console.error("Failed to update screen resolution on backend:", err);
  }

  // Update UI components
  const modeSelect = document.getElementById("mode-select") as HTMLSelectElement;
  if (modeSelect) modeSelect.value = String(config.mode || 1);

  const leftHandedToggle = document.getElementById("left-handed-toggle") as HTMLInputElement;
  if (leftHandedToggle) leftHandedToggle.checked = !!config.decoration?.leftHanded;

  const mouseToggle = document.getElementById("mouse-toggle") as HTMLInputElement;
  if (mouseToggle) mouseToggle.checked = config.osu?.mouse !== false;

  const smokeToggle = document.getElementById("smoke-toggle") as HTMLInputElement;
  if (smokeToggle) smokeToggle.checked = !!config.osu?.toggleSmoke;

  const resDisplay = document.getElementById("resolution-display");
  if (resDisplay) {
    resDisplay.textContent = `${res.width || 1920}x${res.height || 1080}`;
  }

  const maniaKeysSelect = document.getElementById("mania-keys-select") as HTMLSelectElement;
  if (maniaKeysSelect) {
    maniaKeysSelect.value = config.mania?.['4K'] !== false ? '4' : '7';
  }

  // Load all images asynchronously
  const promises = Object.entries(imagePaths).map(async ([key, relPath]) => {
    try {
      const dataUrl = await invoke<string>('read_skin_image', { skinName, relPath });
      const img = new Image();
      img.src = dataUrl;
      await new Promise<void>((resolve) => { 
        img.onload = () => resolve(); 
        img.onerror = () => resolve(); 
      });
      loadedImages.set(key, img);
    } catch (_e) {
      const img = new Image(); 
      img.src = TRANSPARENT_1X1;
      loadedImages.set(key, img);
    }
  });
  
  await Promise.all(promises);
  renderManiaKeysBindingUI();
  updateBindingButtons();
  console.log("[meowverlay] Loaded Skin:", skinName, loadedImages.size, "images");
}

// ─── Status Toast ─────────────────────────────────────────────────────────────
function showToast(msg: string, durationMs = 3000) {
  const el = document.getElementById("status-toast");
  if (!el) return;
  el.textContent = msg;
  el.classList.add("visible");
  setTimeout(() => el.classList.remove("visible"), durationMs);
}

// ─── Lock/Unlock Logic ────────────────────────────────────────────────────────
async function lockOverlay() {
  isLocked = true;
  await appWindow.setIgnoreCursorEvents(true);
  
  closeSettings();
  const toggleBtn = document.getElementById("settings-toggle-btn");
  if (toggleBtn) toggleBtn.style.display = "none";
  document.getElementById("overlay-container")?.classList.add("locked");
  
  showToast("🔒 Locked (Click-through). Press Ctrl+Shift+L to unlock.", 4000);
}

async function unlockOverlay() {
  isLocked = false;
  await appWindow.setIgnoreCursorEvents(false);
  
  const toggleBtn = document.getElementById("settings-toggle-btn");
  if (toggleBtn) toggleBtn.style.display = "flex";
  document.getElementById("overlay-container")?.classList.remove("locked");
  
  openSettings();
  showToast("🔓 Unlocked.", 3000);
}

async function toggleLock() {
  if (isLocked) {
    await unlockOverlay();
  } else {
    await lockOverlay();
  }
}

function openSettings() {
  if (isLocked) return;
  document.getElementById("settings-panel")?.classList.add("visible");
}

function closeSettings() {
  document.getElementById("settings-panel")?.classList.remove("visible");
}

// ─── Draw: osu! Standard ─────────────────────────────────────────────────────
function drawStandard() {
  const isMouse = config.osu?.mouse !== false;
  
  // Background
  const bgImg = isMouse ? loadedImages.get('mousebg') : loadedImages.get('tabletbg');
  if (bgImg) ctx.drawImage(bgImg, 0, 0);

  // Keyboard paw states
  const key1 = config.osu?.key1 || [90];
  const key2 = config.osu?.key2 || [88];
  const k1 = key1.some((k: number) => keysPressed.has(k));
  const k2 = key2.some((k: number) => keysPressed.has(k));

  const waveKey = config.osu?.wave || [];
  const waveActive = waveKey.some((k: number) => keysPressed.has(k));

  // Smoke Logic
  const smokeKey = config.osu?.smoke || [67];
  const isSmokeKeyDown = smokeKey.some((k: number) => keysPressed.has(k));

  if (isSmokeKeyDown && !smokeKeyWasDown) {
    if (config.osu?.toggleSmoke) {
      smokeToggled = !smokeToggled;
    } else {
      smokeToggled = true;
    }
  } else if (!isSmokeKeyDown && smokeKeyWasDown) {
    if (!config.osu?.toggleSmoke) {
      smokeToggled = false;
    }
  }
  smokeKeyWasDown = isSmokeKeyDown;

  // Mouse Arm Tracking (Right Hand)
  const ps = config.mousePaw?.pawStartingPoint || [211, 159];
  const pe = config.mousePaw?.pawEndingPoint || [258, 228];
  const offX = config.decoration?.offsetX || [0, 11];
  const offY = config.decoration?.offsetY || [0, -65];
  const sc = config.decoration?.scalar || [1.0, 1.0];
  const idx = isMouse ? 0 : 1;
  const cx = pe[0] + offX[idx];
  const cy = pe[1] + offY[idx];
  const s = sc[idx];
  const res = config.resolution || { width: 1920, height: 1080 };
  const sw = res.width || 1920;
  const sh = res.height || 1080;
  const nx = Math.max(0, Math.min(1, mousePos.x / sw));
  const ny = Math.max(0, Math.min(1, mousePos.y / sh));
  const rx = isMouse ? 88 : 90;
  const ry = isMouse ? 52 : 55;
  const mx = cx + (nx - 0.5) * rx * s;
  const my = cy + (ny - 0.5) * ry * s;

  // Add Smoke Particle
  if (smokeToggled) {
    smokeParticles.push({
      x: mx,
      y: my,
      alpha: 1.0,
      size: 5 + Math.random() * 4
    });
  }

  // Draw Smoke Trails
  if (smokeParticles.length > 0) {
    ctx.save();
    for (let i = smokeParticles.length - 1; i >= 0; i--) {
      const p = smokeParticles[i];
      p.alpha -= 0.015;
      if (p.alpha <= 0) {
        smokeParticles.splice(i, 1);
        continue;
      }
      ctx.beginPath();
      ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
      ctx.fillStyle = `rgba(140, 140, 150, ${p.alpha * 0.7})`;
      ctx.fill();
    }
    ctx.restore();
  }

  // Left Paw Frame Selection
  let leftPaw = loadedImages.get('osu_up');
  if (waveActive) {
    leftPaw = loadedImages.get('wave');
  } else if (smokeToggled) {
    leftPaw = loadedImages.get('smoke');
  } else if (k1) {
    leftPaw = loadedImages.get('osu_left');
  } else if (k2) {
    leftPaw = loadedImages.get('osu_right');
  }
  
  if (leftPaw) ctx.drawImage(leftPaw, 0, 0);

  // Draw Arm curve
  const x0 = ps[0], y0 = ps[1];
  const c1x = x0 + (mx - x0) * 0.1 - (my - y0) * 0.2;
  const c1y = y0 + (my - y0) * 0.8 + (mx - x0) * 0.2;
  const pawColor = config.osu?.paw || [255, 255, 255];
  const pawEdge = config.osu?.pawEdge || [0, 0, 0];

  ctx.beginPath();
  ctx.moveTo(x0, y0);
  ctx.quadraticCurveTo(c1x, c1y, mx, my);
  ctx.lineCap = 'round'; 
  ctx.lineJoin = 'round';
  ctx.strokeStyle = `rgb(${pawEdge.join(',')})`;
  ctx.lineWidth = 14;
  ctx.stroke();
  ctx.strokeStyle = `rgb(${pawColor.join(',')})`;
  ctx.lineWidth = 8;
  ctx.stroke();

  // Draw Tool (Mouse or Pen)
  const toolImg = isMouse ? loadedImages.get('mouse') : loadedImages.get('tablet');
  if (toolImg && toolImg.width > 1) {
    ctx.drawImage(toolImg, mx - toolImg.width / 2, my - toolImg.height / 2);
  }
}

// ─── Draw: osu! Taiko ─────────────────────────────────────────────────────────
function drawTaiko() {
  const bg = loadedImages.get('taiko_bg'); 
  if (bg) ctx.drawImage(bg, 0, 0);

  const lc = (config.taiko?.leftCentre || [88]).some((k: number) => keysPressed.has(k));
  const lr = (config.taiko?.leftRim || [90]).some((k: number) => keysPressed.has(k));
  const rc = (config.taiko?.rightCentre || [67]).some((k: number) => keysPressed.has(k));
  const rr = (config.taiko?.rightRim || [86]).some((k: number) => keysPressed.has(k));

  let li = loadedImages.get('taiko_leftup');
  if (lr) li = loadedImages.get('taiko_leftrim'); 
  else if (lc) li = loadedImages.get('taiko_leftcentre');
  if (li) ctx.drawImage(li, 0, 0);

  let ri = loadedImages.get('taiko_rightup');
  if (rr) ri = loadedImages.get('taiko_rightrim'); 
  else if (rc) ri = loadedImages.get('taiko_rightcentre');
  if (ri) ctx.drawImage(ri, 0, 0);
}

// ─── Draw: osu! Catch ─────────────────────────────────────────────────────────
function drawCatch() {
  const bg = loadedImages.get('catch_bg'); 
  if (bg) ctx.drawImage(bg, 0, 0);

  const l = (config.catch?.left || [37]).some((k: number) => keysPressed.has(k));
  const r = (config.catch?.right || [39]).some((k: number) => keysPressed.has(k));
  const d = (config.catch?.dash || [16]).some((k: number) => keysPressed.has(k));

  let img = loadedImages.get('catch_up');
  if (d) img = loadedImages.get('catch_dash');
  else if (l && r) img = loadedImages.get('catch_mid');
  else if (l) img = loadedImages.get('catch_left');
  else if (r) img = loadedImages.get('catch_right');
  
  if (img) ctx.drawImage(img, 0, 0);
}

// ─── Draw: osu! Mania ────────────────────────────────────────────────────────
function drawMania() {
  const is4K = config.mania?.['4K'] !== false;
  const bg = is4K ? loadedImages.get('mania_bg_4K') : loadedImages.get('mania_bg_7K');
  if (bg) ctx.drawImage(bg, 0, 0);

  const keys: number[] = is4K ? 
    (config.mania?.key4K || [68, 70, 74, 75]) : 
    (config.mania?.key7K || [83, 68, 70, 32, 74, 75, 76]);

  for (let i = 0; i < keys.length; i++) {
    if (keysPressed.has(keys[i])) {
      const ki = is4K ? loadedImages.get(`key_4K_${i}`) : loadedImages.get(`key_7K_${i}`);
      if (ki) ctx.drawImage(ki, 0, 0);
    }
  }

  let lh = loadedImages.get('mania_leftup');
  let rh = loadedImages.get('mania_rightup');
  if (is4K) {
    if (keysPressed.has(keys[0])) lh = loadedImages.get('mania_left0');
    else if (keysPressed.has(keys[1])) lh = loadedImages.get('mania_left1');
    if (keysPressed.has(keys[2])) rh = loadedImages.get('mania_right0');
    else if (keysPressed.has(keys[3])) rh = loadedImages.get('mania_right1');
  } else {
    if (keysPressed.has(keys[0])) lh = loadedImages.get('mania_left0');
    else if (keysPressed.has(keys[1])) lh = loadedImages.get('mania_left1');
    else if (keysPressed.has(keys[2]) || keysPressed.has(keys[3])) lh = loadedImages.get('mania_left2');
    
    if (keysPressed.has(keys[4])) rh = loadedImages.get('mania_right0');
    else if (keysPressed.has(keys[5])) rh = loadedImages.get('mania_right1');
    else if (keysPressed.has(keys[6])) rh = loadedImages.get('mania_right2');
  }

  if (lh) ctx.drawImage(lh, 0, 0);
  if (rh) ctx.drawImage(rh, 0, 0);
}

// ─── Render Loop ──────────────────────────────────────────────────────────────
function renderLoop() {
  if (ctx && canvas) {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    
    ctx.save();
    // Support Horizontal Mirror for Left Handed Mode
    if (config.decoration?.leftHanded) {
      ctx.translate(canvas.width, 0);
      ctx.scale(-1, 1);
    }

    const mode = config.mode || 1;
    if (mode === 1) drawStandard();
    else if (mode === 2) drawTaiko();
    else if (mode === 3) drawCatch();
    else if (mode === 4) drawMania();

    ctx.restore();
  }
  requestAnimationFrame(renderLoop);
}

// ─── Boot ─────────────────────────────────────────────────────────────────────
window.addEventListener("DOMContentLoaded", async () => {
  canvas = document.getElementById("overlay-canvas") as HTMLCanvasElement;
  ctx = canvas.getContext("2d")!;
  
  const skinSelect = document.getElementById("skin-select") as HTMLSelectElement;
  const modeSelect = document.getElementById("mode-select") as HTMLSelectElement;
  const settingsToggleBtn = document.getElementById("settings-toggle-btn")!;
  const closeSettingsBtn = document.getElementById("close-settings-btn")!;
  const saveConfigBtn = document.getElementById("save-config-btn")!;
  const lockBtn = document.getElementById("lock-btn")!;
  const statusBadge = document.getElementById("global-input-status");

  const leftHandedToggle = document.getElementById("left-handed-toggle") as HTMLInputElement;
  const mouseToggle = document.getElementById("mouse-toggle") as HTMLInputElement;
  const smokeToggle = document.getElementById("smoke-toggle") as HTMLInputElement;
  const maniaKeysSelect = document.getElementById("mania-keys-select") as HTMLSelectElement;

  // ── Drag Window ──
  // Click on background canvas to drag window when settings is closed and unlocked
  canvas.addEventListener("mousedown", async (e) => {
    if (!isLocked && e.button === 0) {
      const panel = document.getElementById("settings-panel");
      if (!panel?.classList.contains("visible")) {
        e.preventDefault();
        await appWindow.startDragging();
      }
    }
  });

  // Settings Panel drag region
  const dragRegions = document.querySelectorAll("[data-tauri-drag-region]");
  dragRegions.forEach(el => {
    el.addEventListener("mousedown", async (e: any) => {
      // Don't drag if clicking buttons, selects, or inputs
      if (e.target.tagName !== "BUTTON" && e.target.tagName !== "SELECT" && e.target.tagName !== "INPUT" && !isLocked && e.button === 0) {
        e.preventDefault();
        await appWindow.startDragging();
      }
    });
  });

  // ── Tab Management ──
  const tabs = document.querySelectorAll('.tab-btn');
  tabs.forEach(tab => {
    tab.addEventListener("click", () => {
      tabs.forEach(t => t.classList.remove("active"));
      tab.classList.add("active");
      
      const panes = document.querySelectorAll('.tab-pane');
      panes.forEach(p => p.classList.remove("active"));
      
      const targetPane = document.getElementById(`tab-${tab.getAttribute('data-tab')}`);
      if (targetPane) targetPane.classList.add("active");
    });
  });

  // ── Settings Open/Close ──
  settingsToggleBtn.addEventListener("click", () => {
    const panel = document.getElementById("settings-panel");
    if (panel?.classList.contains("visible")) {
      closeSettings();
    } else {
      openSettings();
    }
  });

  closeSettingsBtn.addEventListener("click", () => {
    closeSettings();
  });

  // ── Save Configuration ──
  saveConfigBtn.addEventListener("click", async () => {
    try {
      const configStr = JSON.stringify(config, null, 2);
      await invoke('write_skin_config', { skinName: currentSkin, configStr });
      showToast("✔ Configuration saved to disk!");
    } catch (err) {
      console.error(err);
      showToast("❌ Failed to save configuration");
    }
  });

  // ── Lock Overlay ──
  lockBtn.addEventListener("click", async () => {
    await lockOverlay();
  });

  // ── Input Binding Listeners ──
  leftHandedToggle.addEventListener("change", () => {
    if (!config.decoration) config.decoration = {};
    config.decoration.leftHanded = leftHandedToggle.checked;
  });

  mouseToggle.addEventListener("change", () => {
    if (!config.osu) config.osu = {};
    config.osu.mouse = mouseToggle.checked;
  });

  smokeToggle.addEventListener("change", () => {
    if (!config.osu) config.osu = {};
    config.osu.toggleSmoke = smokeToggle.checked;
  });

  maniaKeysSelect.addEventListener("change", () => {
    if (!config.mania) config.mania = {};
    config.mania['4K'] = (maniaKeysSelect.value === '4');
    renderManiaKeysBindingUI();
  });

  // Initialize general Key Bind buttons clicks
  document.querySelectorAll('.key-bind-btn').forEach(btn => {
    const path = btn.getAttribute('data-config-path');
    if (path) {
      btn.addEventListener("click", () => startBinding(btn as HTMLButtonElement, path));
    }
  });

  // ── Local fallbacks (keyboard & mouse) ──
  document.addEventListener("keydown", (e) => {
    if (e.ctrlKey && e.key.toLowerCase() === "r") {
      e.preventDefault();
      loadSkin(currentSkin);
      showToast("✔ Skin reloaded");
      return;
    }
    
    keysPressed.add(e.keyCode);
    
    // Ctrl+Shift+L to toggle lock
    if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "l") {
      e.preventDefault();
      toggleLock();
      return;
    }
    
    if (handleKeyForBinding(e.keyCode)) {
      e.preventDefault();
    }
  });

  document.addEventListener("keyup", (e) => {
    keysPressed.delete(e.keyCode);
  });

  document.addEventListener("mousemove", (e) => {
    if (!globalInputActive) {
      // Local fallback mouse tracking inside window
      const res = config.resolution || { width: 1920, height: 1080 };
      mousePos.x = (e.clientX / 612) * (res.width || 1920);
      mousePos.y = (e.clientY / 354) * (res.height || 1080);
    }
  });

  // ── Skin selector ──
  try {
    const skins = await invoke<string[]>('get_skins');
    skinSelect.innerHTML = "";
    skins.forEach(skin => {
      const opt = document.createElement("option");
      opt.value = skin;
      opt.textContent = skin.charAt(0).toUpperCase() + skin.slice(1);
      skinSelect.appendChild(opt);
    });
    skinSelect.value = currentSkin;
  } catch (err) {
    console.error("Failed to load skins", err);
  }

  skinSelect.addEventListener("change", async () => {
    currentSkin = skinSelect.value;
    await loadSkin(currentSkin);
    showToast(`Loaded skin: ${currentSkin}`);
  });

  // ── Mode selector ──
  modeSelect.addEventListener("change", () => {
    config.mode = parseInt(modeSelect.value, 10);
  });

  // ── Boot sequence ──
  await loadSkin(currentSkin);
  modeSelect.value = String(config.mode || 1);

  // ── Global OS Input Listeners ──
  let gotGlobalEvent = false;
  await listen<any>("input-event", (event) => {
    const { action, key_code } = event.payload;
    if (action === "keydown") {
      keysPressed.add(key_code);
      
      // Global Hotkey: Ctrl + Shift + L
      if (keysPressed.has(17) && keysPressed.has(16) && keysPressed.has(76)) {
        toggleLock();
        return;
      }
      
      if (handleKeyForBinding(key_code)) {
        return;
      }
    } else if (action === "keyup") {
      keysPressed.delete(key_code);
    }

    if (!gotGlobalEvent) {
      gotGlobalEvent = true;
      globalInputActive = true;
      if (statusBadge) {
        statusBadge.textContent = "Active (XWayland)";
        statusBadge.className = "status-badge active";
      }
      showToast("✔ Global input connected", 3000);
      console.log("[meowverlay] Global input connected.");
    }
  });

  await listen<any>("mouse-move", (event) => {
    globalInputActive = true;
    mousePos.x = event.payload.x;
    mousePos.y = event.payload.y;
  });

  // ── Status warning after boot ──
  setTimeout(() => {
    if (!globalInputActive) {
      if (statusBadge) {
        statusBadge.textContent = "Local Only (No Global)";
        statusBadge.className = "status-badge local";
      }
      showToast("⚠ Running in Local Mode. Verify XWayland permissions if global tracking fails.", 6000);
      console.warn("[meowverlay] Global input not active. Verify permissions.");
    }
  }, 2500);

  // Start the render loop
  renderLoop();
});
