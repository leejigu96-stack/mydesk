// 시스템 튜너 — 실제 청소 작업

use sysinfo::{ProcessRefreshKind, RefreshKind, System};

pub fn clean_temp_files() -> Result<String, String> {
    let mut total_freed: u64 = 0;
    let mut count = 0;

    // 사용자 Temp
    if let Ok(temp) = std::env::var("TEMP") {
        let (freed, n) = clean_dir(&temp);
        total_freed += freed;
        count += n;
    }

    // Windows Temp
    let (freed, n) = clean_dir("C:\\Windows\\Temp");
    total_freed += freed;
    count += n;

    let mb = total_freed / 1024 / 1024;
    Ok(format!("{} 파일 정리 / {}MB 회수", count, mb))
}

fn clean_dir(path: &str) -> (u64, u32) {
    let p = std::path::Path::new(path);
    if !p.exists() {
        return (0, 0);
    }
    let mut freed: u64 = 0;
    let mut count: u32 = 0;
    if let Ok(entries) = std::fs::read_dir(p) {
        for e in entries.filter_map(|e| e.ok()) {
            let path = e.path();
            if let Ok(meta) = e.metadata() {
                let size = meta.len();
                if path.is_file() {
                    if std::fs::remove_file(&path).is_ok() {
                        freed += size;
                        count += 1;
                    }
                } else if path.is_dir() {
                    // 자기 자신 폴더(현재 클로드 실행 중)는 건드리지 않음
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !name.starts_with("claude") && !name.contains("MyDesk") {
                        if std::fs::remove_dir_all(&path).is_ok() {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    (freed, count)
}

pub fn kill_zombie_processes() -> Result<String, String> {
    let mut sys = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
    );
    sys.refresh_processes();

    // 좀비로 판단할 프로세스 (빌드 도구가 빌드 끝나고 죽지 않은 경우)
    let zombie_names = ["rustc", "cargo", "node", "linker"];
    let mut killed = 0;
    let our_pid = std::process::id();

    for (pid, proc) in sys.processes() {
        if pid.as_u32() == our_pid {
            continue;
        }
        let name = proc.name().to_lowercase();
        // 좀비 후보 + CPU 0% + 5분 이상 idle
        let is_candidate = zombie_names.iter().any(|n| name.contains(n));
        if is_candidate {
            // 마지막 CPU 사용량 거의 0이면 좀비
            if proc.cpu_usage() < 0.1 && proc.run_time() > 300 {
                if proc.kill() {
                    killed += 1;
                }
            }
        }
    }

    Ok(format!("좀비 {}개 종료", killed))
}

pub fn clear_recycle_bin() -> Result<String, String> {
    let output = std::process::Command::new("powershell")
        .args(&["-Command", "Clear-RecycleBin -Force -ErrorAction SilentlyContinue"])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok("휴지통 비움".to_string())
    } else {
        Err("실패".to_string())
    }
}

pub fn auto_optimize() -> Result<String, String> {
    let mut results = Vec::new();

    if let Ok(r) = clean_temp_files() {
        results.push(format!("임시파일: {}", r));
    }
    if let Ok(r) = kill_zombie_processes() {
        results.push(r);
    }
    if let Ok(r) = clear_recycle_bin() {
        results.push(r);
    }

    Ok(results.join(" / "))
}
