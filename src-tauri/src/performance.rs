// 시스템 성능 모니터링
// RAM / CPU / GPU 사용량 조회

use serde_json::json;
use sysinfo::{System, RefreshKind, CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind};

pub fn get_stats() -> serde_json::Value {
    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything())
            .with_processes(ProcessRefreshKind::everything()),
    );
    sys.refresh_all();

    // MyDesk 자기 자신의 RAM 사용량
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
        "gpu_percent": get_gpu_usage(),  // 추후 구현 (Windows nvapi 또는 WMI)
    })
}

fn get_gpu_usage() -> f32 {
    // TODO: Windows Performance Counter로 GPU 사용량 가져오기
    // 또는 nvml-wrapper로 NVIDIA GPU만 모니터링
    // 지금은 placeholder
    0.0
}
