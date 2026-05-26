// 시스템 성능 + 데이터 수집

use serde_json::json;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, RefreshKind, System};

pub fn get_stats() -> serde_json::Value {
    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything())
            .with_processes(ProcessRefreshKind::everything()),
    );
    sys.refresh_all();

    let our_pid = std::process::id();
    let our_ram = sys
        .process(sysinfo::Pid::from_u32(our_pid))
        .map(|p| p.memory() / 1024 / 1024)
        .unwrap_or(0);

    let total_ram = sys.total_memory() / 1024 / 1024;
    let used_ram = sys.used_memory() / 1024 / 1024;
    let cpu_usage = sys.global_cpu_info().cpu_usage();

    json!({
        "ram_mb_total": total_ram,
        "ram_mb_used": used_ram,
        "ram_mb_mydesk": our_ram,
        "cpu_percent": cpu_usage,
        "gpu_percent": 0.0,  // 추후
    })
}

pub fn count_processes(name_contains: &str) -> u32 {
    let mut sys = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
    );
    sys.refresh_processes();
    sys.processes()
        .values()
        .filter(|p| p.name().to_lowercase().contains(&name_contains.to_lowercase()))
        .count() as u32
}

pub fn count_files_in(path: &str) -> u32 {
    let p = std::path::Path::new(path);
    if !p.exists() {
        return 0;
    }
    std::fs::read_dir(p)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .count() as u32
        })
        .unwrap_or(0)
}
