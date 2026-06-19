use super::h264::{
    encoder_chain, encoder_codec_args_live, finalize_vf, hwaccel_input_args, EncoderKind,
};
use crate::common::MediaError;
use lianli_shared::screen::ScreenInfo;
use parking_lot::Mutex;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::io::AsRawFd;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Try to grow ffmpeg's stdin pipe buffer so a full RGBA frame fits in a single
/// kernel-side buffer. With the default 64 KB pipe size, a 3.7 MB frame
/// fragments into ~58 write() syscalls and ffmpeg rate-limits the writer to
/// pipe-empty cadence; a larger pipe lets us write the whole frame in one syscall
/// and queue ahead by a frame, smoothing out the encoder. We try to fit two
/// frames; if the kernel caps us (EPERM at fs.pipe-max-size, default 1 MB), we
/// settle for whatever size we can get and continue.
fn grow_pipe(fd: i32, frame_bytes: usize) {
    let want = (frame_bytes * 2).max(1 << 22) as libc::c_int;
    let n = unsafe { libc::fcntl(fd, libc::F_SETPIPE_SZ, want) };
    if n < 0 {
        let err = std::io::Error::last_os_error();
        debug!(
            "F_SETPIPE_SZ {} failed: {err}; pipe will use default size",
            want
        );
    } else {
        debug!("pipe buffer sized to {} bytes", n);
    }
}

/// Long-running ffmpeg subprocess that consumes raw RGBA frames on stdin and
/// emits a continuous H.264 NAL stream on stdout. Used by Custom mode for live
/// h264 streaming on devices that accept it.
pub struct LiveH264Encoder {
    child: Child,
    stdin: Option<BufWriter<ChildStdin>>,
    stdout: Option<ChildStdout>,
    frame_bytes: usize,
}

impl LiveH264Encoder {
    pub fn spawn(
        width: u32,
        height: u32,
        fps: f32,
        rotation_deg: u16,
        _screen: &ScreenInfo,
    ) -> Result<Self, MediaError> {
        let fps_int = fps.round().max(1.0) as u32;
        let bitrate = (width as u64 * height as u64 * fps_int as u64 / 4).max(1_000_000);
        let bitrate_str = format!("{bitrate}");
        let fps_str = fps_int.to_string();
        let size_str = format!("{width}x{height}");
        let transpose = match rotation_deg {
            90 => Some("transpose=1"),
            180 => Some("transpose=1,transpose=1"),
            270 => Some("transpose=2"),
            _ => None,
        };

        let mut last_err: Option<String> = None;
        for kind in encoder_chain() {
            match try_spawn(*kind, &size_str, &fps_str, &bitrate_str, transpose) {
                Ok(child) => {
                    info!(
                        "live H.264 encoder: {width}x{height}@{fps_int}fps via {}",
                        kind.name()
                    );
                    return Ok(child);
                }
                Err(e) => {
                    warn!("h264 encoder {} unavailable, trying next: {e}", kind.name());
                    last_err = Some(e);
                }
            }
        }

        Err(MediaError::Ffmpeg(format!(
            "all live H.264 encoders failed; last error: {}",
            last_err.unwrap_or_default()
        )))
    }

    /// Push one raw RGBA frame into the encoder. Returns Err on broken pipe
    /// (encoder died) so the caller can tear down.
    pub fn write_frame(&mut self, rgba: &[u8]) -> Result<(), MediaError> {
        if rgba.len() != self.frame_bytes {
            return Err(MediaError::Ffmpeg(format!(
                "frame size mismatch: got {} bytes, expected {}",
                rgba.len(),
                self.frame_bytes
            )));
        }
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| MediaError::Ffmpeg("encoder stdin already closed".into()))?;
        stdin
            .write_all(rgba)
            .map_err(|e| MediaError::Ffmpeg(format!("write_frame: {e}")))?;
        stdin
            .flush()
            .map_err(|e| MediaError::Ffmpeg(format!("flush: {e}")))?;
        Ok(())
    }

    /// Hand the encoder's stdout to the streaming consumer. Callable once.
    pub fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.stdout.take()
    }
}

impl Drop for LiveH264Encoder {
    fn drop(&mut self) {
        // Closing stdin signals EOF to ffmpeg, which then flushes and exits.
        drop(self.stdin.take());
        let deadline = Instant::now() + Duration::from_millis(500);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) => {
                    if Instant::now() >= deadline {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(e) => {
                    warn!("ffmpeg wait failed: {e}");
                    break;
                }
            }
        }
        if let Err(e) = self.child.kill() {
            warn!("ffmpeg kill failed: {e}");
        }
        let _ = self.child.wait();
    }
}

fn try_spawn(
    kind: EncoderKind,
    size_str: &str,
    fps_str: &str,
    bitrate_str: &str,
    transpose: Option<&str>,
) -> Result<LiveH264Encoder, String> {
    let mut args: Vec<String> = vec!["-loglevel".into(), "error".into()];
    args.extend(hwaccel_input_args(kind));
    args.extend([
        "-f".into(),
        "rawvideo".into(),
        "-pix_fmt".into(),
        "rgba".into(),
        "-s".into(),
        size_str.into(),
        "-r".into(),
        fps_str.into(),
        "-i".into(),
        "pipe:0".into(),
    ]);
    let base_vf = transpose.unwrap_or("");
    let vf = finalize_vf(kind, base_vf);
    if !vf.is_empty() {
        args.extend(["-vf".into(), vf]);
    }
    args.extend(encoder_codec_args_live(kind, fps_str, bitrate_str));
    args.extend(["-color_range".into(), "pc".into()]);
    args.extend(["-an".into(), "-f".into(), "h264".into(), "pipe:1".into()]);

    debug!("live h264 ffmpeg args: {}", args.join(" "));
    let mut child = Command::new("ffmpeg")
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn ffmpeg: {e}"))?;

    std::thread::sleep(Duration::from_millis(200));
    if let Ok(Some(status)) = child.try_wait() {
        let mut err_buf = String::new();
        if let Some(mut stderr) = child.stderr.take() {
            use std::io::Read;
            let _ = stderr.read_to_string(&mut err_buf);
        }
        return Err(format!(
            "ffmpeg exited early ({status}): {}",
            err_buf.trim()
        ));
    }

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "ffmpeg stdin missing".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "ffmpeg stdout missing".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "ffmpeg stderr missing".to_string())?;

    let (w, h) = size_str
        .split_once('x')
        .and_then(|(a, b)| Some((a.parse::<u32>().ok()?, b.parse::<u32>().ok()?)))
        .ok_or_else(|| format!("bad size_str {size_str}"))?;
    let frame_bytes = (w as usize) * (h as usize) * 4;
    grow_pipe(stdin.as_raw_fd(), frame_bytes);

    let probing = Arc::new(AtomicBool::new(true));
    let open_failed = Arc::new(AtomicBool::new(false));
    let probe_log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let probing_clone = Arc::clone(&probing);
    let open_failed_clone = Arc::clone(&open_failed);
    let probe_log_clone = Arc::clone(&probe_log);
    let kind_name = kind.name();
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            if line.is_empty() {
                continue;
            }
            if probing_clone.load(Ordering::Relaxed) {
                if line.contains("Error while opening encoder")
                    || line.contains("Could not open encoder")
                    || line.contains("Failed to initialise")
                    || line.contains("Device creation failed")
                {
                    open_failed_clone.store(true, Ordering::Relaxed);
                }
                probe_log_clone.lock().push(line);
            } else {
                warn!("ffmpeg[{kind_name}]: {line}");
            }
        }
    });

    let probe = vec![0u8; frame_bytes];
    if let Err(e) = stdin.write_all(&probe) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("probe write: {e}"));
    }

    let probe_deadline = Instant::now() + Duration::from_millis(2000);
    loop {
        if open_failed.load(Ordering::Relaxed) {
            let _ = child.kill();
            let _ = child.wait();
            let summary = probe_log
                .lock()
                .iter()
                .find(|l| {
                    !l.contains("Task finished")
                        && !l.contains("Terminating thread")
                        && !l.contains("Nothing was written")
                })
                .cloned()
                .unwrap_or_else(|| format!("{kind_name} encoder open failed"));
            return Err(summary);
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(format!("ffmpeg exited during probe ({status})"));
            }
            Ok(None) => {
                if Instant::now() >= probe_deadline {
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => return Err(format!("try_wait: {e}")),
        }
    }

    probing.store(false, Ordering::Relaxed);

    Ok(LiveH264Encoder {
        child,
        stdin: Some(BufWriter::with_capacity(frame_bytes, stdin)),
        stdout: Some(stdout),
        frame_bytes,
    })
}
