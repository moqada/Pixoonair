use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// see: https://stackoverflow.com/a/78665370
const LOG_PREDICATE: &str = "(eventMessage CONTAINS \"AVCaptureSessionDidStartRunningNotification\" || eventMessage CONTAINS \"AVCaptureSessionDidStopRunningNotification\")";

pub fn start_monitoring<F, G>(on_camera_on: F, on_camera_off: G, stop_flag: Arc<AtomicBool>)
where
    F: Fn() + Send + 'static,
    G: Fn() + Send + 'static,
{
    log::info!("camera monitoring started");
    thread::spawn(move || {
        log::info!("camera monitoring thread started");

        let mut log_process = Command::new("log")
            .arg("stream")
            .arg("--predicate")
            .arg(LOG_PREDICATE)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to execute process");

        if let Some(stdout) = log_process.stdout.take() {
            let reader = BufReader::new(stdout);

            let monitor_thread = thread::spawn(move || {
                let mut lines = reader.lines();
                // 1行目は設定した条件が出力されるだけなのでスキップする
                lines.next();

                for line in lines {
                    match line {
                        Ok(log) => {
                            if log.contains("AVCaptureSessionDidStartRunningNotification") {
                                log::info!("Camera is ON");
                                on_camera_on();
                            } else if log.contains("AVCaptureSessionDidStopRunningNotification") {
                                log::info!("Camera is OFF");
                                on_camera_off();
                            }
                        }
                        Err(e) => {
                            log::error!("Error reading log stream: {}", e);
                        }
                    }
                }
            });
            // プロセスの監視を別スレッドで実行
            thread::spawn({
                let stop_flag = Arc::clone(&stop_flag);
                let mut log_process = log_process;
                move || {
                    while !stop_flag.load(Ordering::Relaxed) {
                        thread::sleep(Duration::from_millis(100));
                    }
                    log::info!("Killing log process...");
                    let kill_result = log_process.kill();
                    match kill_result {
                        Ok(_) => log::info!("Log process killed successfully."),
                        Err(e) => log::error!("Failed to kill log process: {}", e),
                    }
                }
            });
            monitor_thread.join().expect("Log monitor thread panicked");
        }

        log::info!("camera monitoring thread end");
    });
}
