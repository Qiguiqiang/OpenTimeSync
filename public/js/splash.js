const DOM = {
  status: document.getElementById('bootStatusText'),
  caption: document.getElementById('bootCaption'),
  progress: document.getElementById('bootProgressBar')
};

function invokeTauri(command, payload) {
  return window.__TAURI_INTERNALS__.invoke(command, payload);
}

function renderSplash(settings, payload) {
  let status = '正在初始化同步引擎...';
  let caption = 'OPEN TIME SYNC 正在建立首轮高精度时间基线';
  let progress = 18;

  if (!settings.autoSync) {
    status = '正在载入本地时间界面...';
    caption = '当前未开启自动同步，启动后显示本地系统时间';
    progress = 84;
  } else if (settings.syncMode === 'slave' && !payload) {
    status = '正在等待局域网主机响应...';
    caption = '通过局域网主机获取首轮校准结果';
    progress = 34;
  } else if (payload?.calibrationStage === 'stable' && payload?.hasFreshData) {
    status = '首轮校准完成，正在进入主界面...';
    caption = `${payload.sourceLabel} 已建立稳定时间基线`;
    progress = 100;
  } else if (payload?.hasFreshData) {
    status = '已拿到时间样本，继续校准中...';
    caption = `${payload.sourceLabel} 正在稳定偏移和精度结果`;
    progress = 78;
  } else if (payload?.calibrationStage === 'degraded') {
    status = '网络质量一般，使用当前可用结果启动...';
    caption = '未达到理想校准质量，但不会阻塞进入主界面';
    progress = 92;
  } else if (settings.autoSync) {
    progress = 42;
  }

  if (DOM.status) DOM.status.textContent = status;
  if (DOM.caption) DOM.caption.textContent = caption;
  if (DOM.progress) DOM.progress.style.width = `${progress}%`;
}

async function refreshSplash() {
  if (!(window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === 'function')) {
    return;
  }

  try {
    const [settings, payload] = await Promise.all([
      invokeTauri('get_sync_settings'),
      invokeTauri('get_ntp_status').catch(() => null)
    ]);
    renderSplash(settings, payload);
  } catch (_) {}
}

refreshSplash();
setInterval(refreshSplash, 700);
