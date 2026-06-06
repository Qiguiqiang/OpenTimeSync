mod ntp;

use ntp::{query_ntp, remove_outliers, weighted_average, NtpSample, ServerLatency};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::Emitter;
use tauri_plugin_updater::UpdaterExt;

const NTP_SERVERS: &[(&str, &str, &str)] = &[
    ("ntp.tencent.com", "Tencent", "腾讯云"),
    ("ntp.aliyun.com", "Aliyun", "阿里云"),
    ("time.asia.apple.com", "Apple", "Apple Asia"),
    ("time.google.com", "Google", "Google"),
    ("pool.ntp.org", "Pool", "pool.ntp.org"),
];

#[derive(Debug, Clone, Serialize)]
struct NtpTimePayload {
    server_time: u64,
    ntp_offset: f64,
    ntp_rtt: i64,
    ntp_server: String,
    server_latencies: HashMap<String, ServerLatency>,
}

#[derive(Debug, Clone, Serialize)]
struct ActiveServer {
    host: String,
    name: String,
    label: String,
}

struct AppState {
    active_server: Mutex<ActiveServer>,
    last_payload: Mutex<Option<NtpTimePayload>>,
    cycle_count: Mutex<u64>,
    auto_sync: Mutex<bool>,
    last_sync_cycle: Mutex<u64>,
    sync_interval_secs: Mutex<u64>,
    update_status: Mutex<UpdateStatusPayload>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateStatusPayload {
    phase: String,
    current_version: String,
    version: Option<String>,
    message: String,
    downloaded_bytes: Option<u64>,
    total_bytes: Option<u64>,
}

#[allow(non_snake_case)]
#[repr(C)]
struct SYSTEMTIME {
    wYear: u16, wMonth: u16, wDayOfWeek: u16,
    wDay: u16, wHour: u16, wMinute: u16,
    wSecond: u16, wMilliseconds: u16,
}

#[cfg(windows)]
extern "system" {
    fn SetSystemTime(lpSystemTime: *const SYSTEMTIME) -> i32;
    fn GetCurrentProcess() -> isize;
    fn OpenProcessToken(hProcess: isize, dwDesiredAccess: u32, phToken: &mut isize) -> i32;
    fn CloseHandle(hObject: isize) -> i32;
    fn LookupPrivilegeValueW(lpSystemName: *const u16, lpName: *const u16, lpLuid: &mut i64) -> i32;
    fn AdjustTokenPrivileges(
        hToken: isize, bDisableAll: i32, lpNewState: *const TOKEN_PRIVILEGES,
        cbBuffer: u32, lpPreviousState: *mut TOKEN_PRIVILEGES, cbReturn: &mut u32,
    ) -> i32;
}

#[allow(non_snake_case)]
#[repr(C)]
struct LUID_AND_ATTRIBUTES { luid: i64, attributes: u32 }

#[allow(non_snake_case)]
#[repr(C)]
struct TOKEN_PRIVILEGES {
    privilege_count: u32,
    privileges: [LUID_AND_ATTRIBUTES; 1],
}

const SE_SYSTEMTIME_NAME: &str = "SeSystemtimePrivilege\0";
const TOKEN_QUERY: u32 = 0x0008;
const TOKEN_ADJUST_PRIVILEGES: u32 = 0x0020;
const SE_PRIVILEGE_ENABLED: u32 = 0x00000002;

#[cfg(windows)]
fn enable_privilege() -> Result<(), String> {
    unsafe {
        let h = GetCurrentProcess();
        let mut token: isize = 0;
        if OpenProcessToken(h, TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut token) == 0 {
            return Err("无法打开进程令牌".to_string());
        }
        let mut luid: i64 = 0;
        let name = SE_SYSTEMTIME_NAME.encode_utf16().collect::<Vec<_>>();
        if LookupPrivilegeValueW(std::ptr::null(), name.as_ptr(), &mut luid) == 0 {
            CloseHandle(token);
            return Err("无法查询特权".to_string());
        }
        let tp = TOKEN_PRIVILEGES {
            privilege_count: 1,
            privileges: [LUID_AND_ATTRIBUTES { luid, attributes: SE_PRIVILEGE_ENABLED }],
        };
        let mut prev: TOKEN_PRIVILEGES = std::mem::zeroed();
        let mut ret: u32 = 0;
        AdjustTokenPrivileges(
            token, 0, &tp,
            std::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
            &mut prev, &mut ret,
        );
        CloseHandle(token);
        Ok(())
    }
}

#[cfg(windows)]
fn set_windows_system_time(server_time_ms: u64) -> Result<(), String> {
    let total_secs = server_time_ms / 1000;
    let millis = (server_time_ms % 1000) as u16;
    let seconds = (total_secs % 60) as u16;
    let minutes = ((total_secs / 60) % 60) as u16;
    let hours = ((total_secs / 3600) % 24) as u16;
    let days = total_secs / 86400;

    let mut y = 1970i64;
    let mut rem = days as i64;
    loop {
        let diy = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
        if rem < diy { break; }
        rem -= diy; y += 1;
    }
    let md: [i64; 12] = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
        [31,29,31,30,31,30,31,31,30,31,30,31]
    } else {
        [31,28,31,30,31,30,31,31,30,31,30,31]
    };
    let mut mon: u16 = 1;
    for &d in md.iter() {
        if rem < d { break; }
        rem -= d; mon += 1;
    }

    if let Err(e) = enable_privilege() {
        return Err(format!("权限不足: {}", e));
    }

    let st = SYSTEMTIME {
        wYear: y as u16, wMonth: mon, wDayOfWeek: 0, wDay: (rem + 1) as u16,
        wHour: hours, wMinute: minutes, wSecond: seconds, wMilliseconds: millis,
    };

    unsafe {
        if SetSystemTime(&st) == 0 {
            Err("设置系统时间失败，请以管理员身份运行此程序".to_string())
        } else {
            Ok(())
        }
    }
}

#[cfg(not(windows))]
fn set_windows_system_time(_server_time_ms: u64) -> Result<(), String> {
    Err("仅 Windows 支持系统时间同步".to_string())
}

#[tauri::command]
fn ping() -> String {
    "pong".to_string()
}

#[tauri::command]
fn sync_system_time(state: tauri::State<Arc<AppState>>) -> Result<String, String> {
    let payload = state.last_payload.lock().unwrap().clone();
    match payload {
        Some(p) => {
            set_windows_system_time(p.server_time)?;
            *state.last_sync_cycle.lock().unwrap() = *state.cycle_count.lock().unwrap();
            Ok("系统时间已同步".to_string())
        }
        None => Err("尚无 NTP 数据".to_string()),
    }
}

#[tauri::command]
fn set_auto_sync(state: tauri::State<Arc<AppState>>, enabled: bool) {
    *state.auto_sync.lock().unwrap() = enabled;
}

#[tauri::command]
fn get_auto_sync(state: tauri::State<Arc<AppState>>) -> bool {
    *state.auto_sync.lock().unwrap()
}

#[tauri::command]
fn set_sync_interval(state: tauri::State<Arc<AppState>>, seconds: u64) {
    *state.sync_interval_secs.lock().unwrap() = seconds.max(5).min(3600);
}

#[tauri::command]
fn get_sync_interval(state: tauri::State<Arc<AppState>>) -> u64 {
    *state.sync_interval_secs.lock().unwrap()
}

#[tauri::command]
fn set_ntp_server(state: tauri::State<Arc<AppState>>, server: String) -> bool {
    for s in NTP_SERVERS {
        if s.0 == server {
            let mut active = state.active_server.lock().unwrap();
            active.host = s.0.to_string();
            active.name = s.1.to_string();
            active.label = s.2.to_string();
            return true;
        }
    }
    false
}

#[tauri::command]
fn get_ntp_status(state: tauri::State<Arc<AppState>>) -> Option<NtpTimePayload> {
    state.last_payload.lock().unwrap().clone()
}

#[tauri::command]
fn minimize_window(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
fn maximize_window(window: tauri::Window) {
    if let Ok(true) = window.is_maximized() {
        let _ = window.unmaximize();
    } else {
        let _ = window.maximize();
    }
}

#[tauri::command]
fn close_window(window: tauri::Window) {
    let _ = window.close();
}

#[tauri::command]
fn start_drag(window: tauri::Window) {
    let _ = window.start_dragging();
}

#[tauri::command]
fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn set_update_status(
    state: &Arc<AppState>,
    phase: impl Into<String>,
    current_version: impl Into<String>,
    version: Option<String>,
    message: impl Into<String>,
    downloaded_bytes: Option<u64>,
    total_bytes: Option<u64>,
) -> UpdateStatusPayload {
    let status = UpdateStatusPayload {
        phase: phase.into(),
        current_version: current_version.into(),
        version,
        message: message.into(),
        downloaded_bytes,
        total_bytes,
    };
    *state.update_status.lock().unwrap() = status.clone();
    status
}

fn format_update_error(err: impl std::fmt::Display) -> String {
    let msg = err.to_string();
    if msg.contains("signature") {
        "更新签名无效，请先检查发布签名配置".to_string()
    } else if msg.contains("ReleaseNotFound") {
        "未找到可用更新".to_string()
    } else {
        msg
    }
}

#[tauri::command]
fn get_update_status(state: tauri::State<Arc<AppState>>) -> UpdateStatusPayload {
    state.update_status.lock().unwrap().clone()
}

#[tauri::command]
async fn check_for_update(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<UpdateStatusPayload, String> {
    let app_state = state.inner().clone();
    let current_version = app.package_info().version.to_string();

    set_update_status(
        &app_state,
        "checking",
        current_version.clone(),
        None,
        "正在检查更新...",
        None,
        None,
    );

    let updater = app
        .updater_builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(format_update_error)?;

    match updater.check().await.map_err(format_update_error)? {
        Some(update) => Ok(set_update_status(
            &app_state,
            "available",
            current_version,
            Some(update.version.clone()),
            format!("发现新版本 v{}，点击下载并安装", update.version),
            None,
            None,
        )),
        None => Ok(set_update_status(
            &app_state,
            "upToDate",
            current_version,
            None,
            "已是最新版本",
            None,
            None,
        )),
    }
}

#[tauri::command]
fn install_available_update(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let app_state = state.inner().clone();
    let current_version = app.package_info().version.to_string();

    {
        let status = app_state.update_status.lock().unwrap().clone();
        if matches!(status.phase.as_str(), "checking" | "downloading" | "installing") {
            return Err("更新任务正在进行中".to_string());
        }
    }

    set_update_status(
        &app_state,
        "downloading",
        current_version.clone(),
        None,
        "正在准备下载安装...",
        Some(0),
        None,
    );

    tauri::async_runtime::spawn(async move {
        let result: Result<(), String> = async {
            let updater = app
                .updater_builder()
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(format_update_error)?;

            let update = updater
                .check()
                .await
                .map_err(format_update_error)?
                .ok_or_else(|| "已是最新版本".to_string())?;

            let version = update.version.clone();
            let mut downloaded_bytes = 0u64;

            set_update_status(
                &app_state,
                "downloading",
                current_version.clone(),
                Some(version.clone()),
                format!("正在下载 v{}", version),
                Some(0),
                None,
            );

            update
                .download_and_install(
                    |chunk_length, content_length| {
                        downloaded_bytes += chunk_length as u64;
                        let message = match content_length {
                            Some(total) if total > 0 => {
                                let progress =
                                    (downloaded_bytes as f64 / total as f64 * 100.0).round();
                                format!("正在下载 v{} ({}%)", version, progress)
                            }
                            _ => format!("正在下载 v{}", version),
                        };
                        set_update_status(
                            &app_state,
                            "downloading",
                            current_version.clone(),
                            Some(version.clone()),
                            message,
                            Some(downloaded_bytes),
                            content_length,
                        );
                    },
                    || {
                        set_update_status(
                            &app_state,
                            "installing",
                            current_version.clone(),
                            Some(version.clone()),
                            format!("下载完成，正在安装 v{}", version),
                            None,
                            None,
                        );
                    },
                )
                .await
                .map_err(format_update_error)?;

            set_update_status(
                &app_state,
                "installing",
                current_version.clone(),
                Some(version.clone()),
                format!("安装程序已启动，正在应用 v{}", version),
                None,
                None,
            );

            Ok(())
        }
        .await;

        if let Err(err) = result {
            set_update_status(
                &app_state,
                "error",
                current_version,
                None,
                format!("更新失败：{}", err),
                None,
                None,
            );
        }
    });

    Ok(())
}

fn run_ntp_loop(app_handle: tauri::AppHandle, app_state: Arc<AppState>) {
    std::thread::spawn(move || {
        loop {
            let active_host = app_state.active_server.lock().unwrap().host.clone();

            let mut server_latencies = HashMap::new();
            let mut active_samples: Vec<NtpSample> = Vec::new();

            for (host, _name, _label) in NTP_SERVERS {
                match query_ntp(host) {
                    Ok(sample) => {
                        server_latencies.insert(
                            host.to_string(),
                            ServerLatency {
                                rtt: sample.rtt.round() as i64,
                                status: "ok".to_string(),
                            },
                        );
                        if *host == active_host {
                            active_samples.push(sample);
                        }
                    }
                    Err(_) => {
                        server_latencies.insert(
                            host.to_string(),
                            ServerLatency {
                                rtt: -1,
                                status: "timeout".to_string(),
                            },
                        );
                    }
                }
            }

            let ntp_offset = if !active_samples.is_empty() {
                let filtered = remove_outliers(&active_samples, 0.1);
                weighted_average(&filtered)
            } else {
                0.0
            };

            let ntp_rtt = if active_samples.is_empty() {
                -1.0
            } else {
                active_samples.iter().map(|s| s.rtt).fold(f64::MAX, |a, b| a.min(b))
            };

            let corrected_time = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as f64 + ntp_offset) as u64;

            let payload = NtpTimePayload {
                server_time: corrected_time,
                ntp_offset,
                ntp_rtt: if ntp_rtt.is_finite() && ntp_rtt > 0.0 && ntp_rtt < 1_000_000.0 {
                    ntp_rtt.round() as i64
                } else {
                    -1
                },
                ntp_server: active_host,
                server_latencies,
            };

            let cycle = *app_state.cycle_count.lock().unwrap();
            *app_state.last_payload.lock().unwrap() = Some(payload.clone());
            *app_state.cycle_count.lock().unwrap() = cycle + 1;

            if *app_state.auto_sync.lock().unwrap() {
                let last_sync = *app_state.last_sync_cycle.lock().unwrap();
                let interval_secs = *app_state.sync_interval_secs.lock().unwrap();
                let cooldown = (interval_secs + 1) / 2;
                if cycle >= last_sync + cooldown {
                    if let Some(p) = app_state.last_payload.lock().unwrap().clone() {
                        let _ = set_windows_system_time(p.server_time);
                        *app_state.last_sync_cycle.lock().unwrap() = cycle;
                    }
                }
            }

            let _ = app_handle.emit("ntp-time", &payload);
            std::thread::sleep(Duration::from_secs(2));
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = Arc::new(AppState {
        active_server: Mutex::new(ActiveServer {
            host: NTP_SERVERS[0].0.to_string(),
            name: NTP_SERVERS[0].1.to_string(),
            label: NTP_SERVERS[0].2.to_string(),
        }),
        last_payload: Mutex::new(None),
        cycle_count: Mutex::new(0),
        auto_sync: Mutex::new(false),
        last_sync_cycle: Mutex::new(0),
        sync_interval_secs: Mutex::new(30),
        update_status: Mutex::new(UpdateStatusPayload {
            phase: "idle".to_string(),
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            version: None,
            message: String::new(),
            downloaded_bytes: None,
            total_bytes: None,
        }),
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![
            ping,
            get_version,
            sync_system_time,
            set_auto_sync,
            get_auto_sync,
            set_sync_interval,
            get_sync_interval,
            set_ntp_server,
            get_ntp_status,
            minimize_window,
            maximize_window,
            close_window,
            start_drag,
            get_update_status,
            check_for_update,
            install_available_update,
        ])
        .setup(move |app| {
            run_ntp_loop(app.handle().clone(), app_state.clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
