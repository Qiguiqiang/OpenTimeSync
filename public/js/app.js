const DOM = {
  hours: document.getElementById('timeHours'),
  mins: document.getElementById('timeMins'),
  secs: document.getElementById('timeSecs'),
  ms: document.getElementById('currentMs'),
  utcDisplay: document.getElementById('utcDisplay'),
  tzDisplay: document.getElementById('tzDisplay'),
  offsetDisplay: document.getElementById('offsetDisplay'),
  rttDisplay: document.getElementById('rttDisplay'),
  offsetValue: document.getElementById('offsetValue'),
  rttValue: document.getElementById('rttValue'),
  precisionTier: document.getElementById('precisionTier'),
  precisionError: document.getElementById('precisionError'),
  sampleCount: document.getElementById('sampleCount'),
  ntpCount: document.getElementById('ntpCount'),
  statusDot: document.getElementById('statusDot'),
  statusLabel: document.getElementById('statusLabel'),
  tzPanel: document.getElementById('tzPanel')
};

const TIMEZONES = [
  { value: 'UTC', label: 'UTC+0', name: 'UTC' },
  { value: 'Asia/Shanghai', label: 'UTC+8', name: 'Shanghai' },
  { value: 'Asia/Tokyo', label: 'UTC+9', name: 'Tokyo' },
  { value: 'Asia/Singapore', label: 'UTC+8', name: 'Singapore' },
  { value: 'Asia/Dubai', label: 'UTC+4', name: 'Dubai' },
  { value: 'Asia/Kolkata', label: 'UTC+5:30', name: 'Kolkata' },
  { value: 'Europe/Moscow', label: 'UTC+3', name: 'Moscow' },
  { value: 'Europe/Berlin', label: 'UTC+1', name: 'Berlin' },
  { value: 'Europe/London', label: 'UTC+0', name: 'London' },
  { value: 'America/New_York', label: 'UTC-5', name: 'New York' },
  { value: 'America/Chicago', label: 'UTC-6', name: 'Chicago' },
  { value: 'America/Los_Angeles', label: 'UTC-8', name: 'Los Angeles' },
  { value: 'Pacific/Auckland', label: 'UTC+13', name: 'Auckland' },
  { value: 'Australia/Sydney', label: 'UTC+11', name: 'Sydney' }
];

const State = {
  ws: null,
  offset: 0,
  samples: [],
  maxSamples: 20,
  isSynced: false,
  syncBase: 0,
  perfBase: 0,
  offsetStd: 0,
  rtt: 0,
  timezone: '',
  ntpServerCount: 5
};

// ── 标准差计算 ──
function calculateStdDev(arr) {
  if (arr.length < 2) return 0;
  const mean = arr.reduce((a, b) => a + b, 0) / arr.length;
  const variance = arr.reduce((sum, v) => sum + Math.pow(v - mean, 2), 0) / arr.length;
  return Math.sqrt(variance);
}

// ── 精度等级（基于 offsetStd） ──
function getPrecisionTier() {
  const { offsetStd } = State;
  if (offsetStd < 2)   return 'S+';
  if (offsetStd < 5)   return 'S';
  if (offsetStd < 10)  return 'S-';
  if (offsetStd < 30)  return 'A';
  if (offsetStd < 50)  return 'B';
  if (offsetStd < 100) return 'C';
  return 'D';
}

function getPrecisionClass(tier) {
  if (tier === 'S+' || tier === 'S') return 'good';
  if (tier === 'S-' || tier === 'A') return 'warning';
  return 'danger';
}

// ── WebSocket 连接 ──
function connect() {
  const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
  State.ws = new WebSocket(`${proto}//${location.host}`);

  State.ws.onopen = () => {
    setStatus('connecting', 'CONNECTED');
    startRttMeasurement();
  };

  State.ws.onmessage = (e) => {
    const msg = JSON.parse(e.data);
    if (msg.type === 'time') handleTime(msg);
    if (msg.type === 'timeResponse') handleTimeResponse(msg);
  };

  State.ws.onclose = () => {
    setStatus('error', 'DISCONNECTED');
    setTimeout(connect, 3000);
  };

  State.ws.onerror = () => {};
}

function send(data) {
  if (State.ws && State.ws.readyState === WebSocket.OPEN) {
    State.ws.send(JSON.stringify(data));
  }
}

// ── 广播模式：服务器推送时间 ──
function handleTime(msg) {
  const T2 = msg.serverTime;
  const T3 = Date.now();
  const offset = T2 - T3;

  State.samples.push(offset);
  if (State.samples.length > State.maxSamples) State.samples.shift();

  calculateOffset();
  updateUI();
}

// ── RTT 响应处理 ──
function handleTimeResponse(msg) {
  const rtt = Date.now() - msg.t1;
  State.rtt = rtt;
}

// ── 周期性 RTT 测量（每30秒） ──
let rttInterval = null;
function startRttMeasurement() {
  // 立即测量一次
  send({ type: 'getTime', t1: Date.now() });

  // 每30秒测量一次
  rttInterval = setInterval(() => {
    send({ type: 'getTime', t1: Date.now() });
  }, 30000);
}

function calculateOffset() {
  if (State.samples.length < 3) { State.isSynced = false; return; }

  // 过滤异常值（去掉最高/最低 10%）
  const sorted = [...State.samples].sort((a, b) => a - b);
  const cutoff = Math.floor(sorted.length * 0.1);
  const filtered = sorted.slice(cutoff, sorted.length - cutoff);

  // 简单平均
  State.offset = filtered.reduce((a, b) => a + b, 0) / filtered.length;

  // 计算标准差
  State.offsetStd = calculateStdDev(filtered);

  State.syncBase = Date.now() + State.offset;
  State.perfBase = performance.now();
  State.isSynced = true;
}

function getSyncTime() { return State.syncBase + (performance.now() - State.perfBase); }

// ── UI 更新 ──
function updateUI() {
  DOM.offsetDisplay.textContent = `DIFF: ${State.offset.toFixed(2)}ms`;
  DOM.rttDisplay.textContent = `PING: ${State.rtt > 0 ? State.rtt + 'ms' : '--'}`;

  DOM.offsetValue.textContent = State.offset.toFixed(2);
  const absOffset = Math.abs(State.offset);
  DOM.offsetValue.className = 'stat-value ' + cls(absOffset, 5, 20);
  DOM.rttValue.textContent = State.rtt > 0 ? State.rtt : '--';
  DOM.rttValue.className = 'stat-value ' + (State.rtt > 0 ? cls(State.rtt, 50, 100) : '');

  // 精度等级（基于 offsetStd）
  const tier = getPrecisionTier();
  DOM.precisionTier.textContent = tier;
  DOM.precisionTier.className = 'stat-value ' + getPrecisionClass(tier);
  DOM.precisionError.textContent = `±${State.offsetStd.toFixed(2)}ms`;
  DOM.sampleCount.textContent = State.samples.length;

  setStatus('synced', `SYNCED ±${State.offset.toFixed(1)}ms`);
}

function cls(v, warn, danger) { return v < warn ? 'good' : v < danger ? 'warning' : 'danger'; }

function setStatus(state, label) {
  DOM.statusDot.className = 'status-dot ' + state;
  DOM.statusLabel.textContent = label;
}

// ── 时区功能 ──
function initTimezone() {
  const saved = localStorage.getItem('timesync-tz');
  const detected = Intl.DateTimeFormat().resolvedOptions().timeZone;
  State.timezone = saved || detected;
  renderTzList();
}

function setTz(tz) {
  State.timezone = tz;
  localStorage.setItem('timesync-tz', tz);
  DOM.tzPanel.classList.remove('open');
  renderTzList();
}

function renderTzList() {
  const current = TIMEZONES.find(t => t.value === State.timezone);
  const label = current ? `${current.label} ${current.name}` : State.timezone;
  DOM.tzDisplay.textContent = label;

  DOM.tzPanel.innerHTML = TIMEZONES.map(t => {
    const active = t.value === State.timezone ? ' active' : '';
    return `<div class="tz-item${active}" data-tz="${t.value}">${t.label} ${t.name}</div>`;
  }).join('');

  DOM.tzPanel.querySelectorAll('.tz-item').forEach(el => {
    el.addEventListener('click', () => setTz(el.dataset.tz));
  });
}

function toggleTzPanel() {
  DOM.tzPanel.classList.toggle('open');
}

// 点击外部关闭时区面板
document.addEventListener('click', (e) => {
  if (!e.target.closest('.tz-selector')) {
    DOM.tzPanel.classList.remove('open');
  }
});

// ── 渲染循环 ──
let lastSec = -1;
function renderLoop() {
  if (State.isSynced) {
    const now = getSyncTime();
    const d = new Date(now);
    const ms = String(d.getMilliseconds()).padStart(3, '0');

    // 使用时区格式化
    const tzParts = getTzParts(now);
    DOM.hours.textContent = tzParts.h;
    DOM.mins.textContent = tzParts.m;
    const sec = parseInt(tzParts.s);
    DOM.secs.textContent = String(sec).padStart(2, '0');
    if (sec !== lastSec) {
      DOM.secs.classList.remove('pulse');
      void DOM.secs.offsetWidth;
      DOM.secs.classList.add('pulse');
      lastSec = sec;
    }
    DOM.ms.textContent = `.${ms}`;

    // UTC 显示
    DOM.utcDisplay.textContent = `UTC: ${d.toISOString().replace('T', ' ').substring(0, 23)}`;
  }
  requestAnimationFrame(renderLoop);
}

function getTzParts(timestamp) {
  const d = new Date(timestamp);
  try {
    const parts = new Intl.DateTimeFormat('en-US', {
      timeZone: State.timezone,
      hour12: false,
      hour: '2-digit', minute: '2-digit', second: '2-digit'
    }).formatToParts(d);
    const h = parts.find(p => p.type === 'hour')?.value || '00';
    const m = parts.find(p => p.type === 'minute')?.value || '00';
    const s = parts.find(p => p.type === 'second')?.value || '00';
    return { h, m, s };
  } catch {
    return {
      h: String(d.getHours()).padStart(2, '0'),
      m: String(d.getMinutes()).padStart(2, '0'),
      s: String(d.getSeconds()).padStart(2, '0')
    };
  }
}

// ── 初始化 ──
initTimezone();
connect();
requestAnimationFrame(renderLoop);
