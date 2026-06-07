use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use rand::seq::SliceRandom;

use rodio::{Decoder, Sink, Source, OutputStream, OutputStreamHandle};
use cpal::traits::DeviceTrait;

use crate::error::PlaybackError;
use crate::filter::FilterProfile;
use crate::pipeline::PipelineSource;
use crate::spectrum::{BAND_COUNT, SpectrumOutput};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackMode {
    Sequential,
    LoopOne,
    LoopAll,
    Shuffle,
}

fn is_bluetooth_device(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("bluetooth")
        || lower.contains("srs")
        || lower.contains("jbl")
        || lower.contains("索尼")
        || lower.contains("sony")
        || lower.contains("bose")
        || lower.contains("airpods")
        || lower.contains("galaxy buds")
        || lower.contains("mi band")
        || lower.contains("小米")
        || lower.contains("huawei")
        || lower.contains("蓝牙")
}

struct Inner {
    stream_sink: Option<(OutputStream, Sink)>,
    file_path: Option<PathBuf>,
    duration: f64,
    sample_rate: u32,
    play_start: Option<Instant>,
    pause_start: Option<Instant>,
    total_paused: Duration,
    paused: bool,
    seek_offset: f64,
    decoder_sample_rate: Option<u32>,
    control: Option<crate::pipeline::PipelineControl>,
    queue: Vec<PathBuf>,
    original_queue: Vec<PathBuf>,
    mode: PlaybackMode,
    output_device: Option<String>,
    meter: Option<Arc<AtomicU32>>,
    spectrum_output: Option<Arc<SpectrumOutput>>,
}

impl std::fmt::Debug for Inner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inner")
            .field("file_path", &self.file_path)
            .field("duration", &self.duration)
            .field("sample_rate", &self.sample_rate)
            .field("play_start", &self.play_start)
            .field("pause_start", &self.pause_start)
            .field("total_paused", &self.total_paused)
            .field("paused", &self.paused)
            .field("decoder_sample_rate", &self.decoder_sample_rate)
            .finish()
    }
}

impl Inner {
    fn new() -> Self {
        Self {
            stream_sink: None,
            file_path: None,
            duration: 0.0,
            sample_rate: 0,
            play_start: None,
            pause_start: None,
            total_paused: Duration::ZERO,
            paused: false,
            seek_offset: 0.0,
            decoder_sample_rate: None,
            control: None,
            queue: Vec::new(),
            original_queue: Vec::new(),
            mode: PlaybackMode::Sequential,
            output_device: None,
            meter: None,
            spectrum_output: None,
        }
    }

    fn elapsed_secs(&self) -> f64 {
        let play_start = match self.play_start {
            Some(ps) => ps,
            None => return self.seek_offset,
        };
        let total_elapsed = play_start.elapsed();
        let mut net = total_elapsed - self.total_paused;
        if self.paused {
            if let Some(pause_start) = self.pause_start {
                net -= pause_start.elapsed();
            }
        }
        self.seek_offset + net.as_secs_f64()
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    inner: Arc<Mutex<Inner>>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::new())),
        }
    }

    fn stop_inner(inner: &mut Inner) {
        if let Some((_stream, sink)) = inner.stream_sink.take() {
            sink.stop();
        }
        inner.play_start = None;
        inner.pause_start = None;
        inner.total_paused = Duration::ZERO;
        inner.paused = false;
        inner.seek_offset = 0.0;
        inner.control = None;
        inner.queue.clear();
    }

    fn start_playback_inner(
        &self,
        inner: &mut std::sync::MutexGuard<'_, Inner>,
        offset_secs: f64,
    ) -> Result<(), PlaybackError> {
        let file_path = inner
            .file_path
            .as_ref()
            .ok_or_else(|| PlaybackError::Other("No file set for playback".into()))?;

        let meter = Arc::new(AtomicU32::new(0));
        let output = Arc::new(SpectrumOutput::new());
        let source = match PipelineSource::new(
            file_path,
            FilterProfile::NOS,
            0.0,
            meter.clone(),
            output.clone(),
            inner.sample_rate,
        ) {
            Ok(s) => s,
            Err(e) => {
                return Err(PlaybackError::Other(e.to_string()));
            }
        };

        inner.control = Some(source.control());
        inner.meter = Some(meter);
        inner.spectrum_output = Some(output);
        inner.sample_rate = source.sample_rate();
        inner.duration = source.duration_secs();

        let source = source.skip_duration(Duration::from_secs_f64(offset_secs));

        let (stream, handle) = self.build_output_stream(inner)?;
        let sink = Sink::try_new(&handle).map_err(|_| PlaybackError::NoDevice)?;

        let bt = inner.output_device.as_ref().map_or(false, |n| is_bluetooth_device(n));
        if bt {
            sink.append(source.buffered());
        } else {
            sink.append(source);
        }
        sink.play();

        inner.stream_sink = Some((stream, sink));
        inner.play_start = Some(Instant::now());
        inner.pause_start = None;
        inner.total_paused = Duration::ZERO;
        inner.paused = false;
        inner.seek_offset = offset_secs;

        Ok(())
    }

    fn build_output_stream(
        &self,
        inner: &mut std::sync::MutexGuard<'_, Inner>,
    ) -> Result<(OutputStream, OutputStreamHandle), PlaybackError> {
        if let Some(ref dev_name) = inner.output_device {
            match crate::device::find_device_by_name(dev_name) {
                Some(device) => {
                    match OutputStream::try_from_device(&device) {
                        Ok(pair) => return Ok(pair),
                        Err(e) => {
                            eprintln!("[Device] 无法使用设备 '{}': {}, 回退到默认设备", dev_name, e);
                            inner.output_device = None;
                        }
                    }
                }
                None => {
                    eprintln!("[Device] 找不到设备 '{}', 回退到默认设备", dev_name);
                    inner.output_device = None;
                }
            }
        }

        OutputStream::try_default().map_err(|_| PlaybackError::NoDevice)
    }

    pub fn play<P: AsRef<Path>>(&self, path: P) -> Result<(), PlaybackError> {
        let path_ref = path.as_ref();

        if !path_ref.exists() {
            return Err(PlaybackError::Other(format!("文件不存在: {}", path_ref.display())));
        }
        if !path_ref.is_file() {
            return Err(PlaybackError::Other(format!("不是文件: {}", path_ref.display())));
        }

        let mut inner = self.inner.lock().unwrap();

        inner.file_path = Some(path_ref.to_path_buf());
        inner.duration = 0.0;
        inner.sample_rate = 44100;

        if inner.stream_sink.is_some() {
            Self::stop_inner(&mut inner);
            inner.file_path = Some(path_ref.to_path_buf());
        }

        let result = self.start_playback_inner(&mut inner, 0.0);
        result?;
        Ok(())
    }

    pub fn pause(&self) {
        let mut inner = self.inner.lock().unwrap();
        if inner.paused {
            return;
        }
        if let Some((_stream, sink)) = inner.stream_sink.as_ref() {
            sink.pause();
            inner.pause_start = Some(Instant::now());
            inner.paused = true;
        }
    }

    pub fn resume(&self) {
        let mut inner = self.inner.lock().unwrap();
        if !inner.paused {
            return;
        }
        if let Some((_stream, sink)) = inner.stream_sink.as_ref() {
            sink.play();
            if let Some(pause_start) = inner.pause_start {
                inner.total_paused += pause_start.elapsed();
            }
            inner.pause_start = None;
            inner.paused = false;
        }
    }

    pub fn stop(&self) {
        let mut inner = self.inner.lock().unwrap();
        Self::stop_inner(&mut inner);
        inner.file_path = None;
        inner.duration = 0.0;
        inner.sample_rate = 0;
        inner.play_start = None;
        inner.total_paused = Duration::ZERO;
        inner.paused = false;
        inner.seek_offset = 0.0;
    }

    pub fn state(&self) -> PlaybackState {
        let inner = self.inner.lock().unwrap();
        match &inner.stream_sink {
            None => PlaybackState::Stopped,
            Some(_) => {
                if inner.paused {
                    PlaybackState::Paused
                } else {
                    PlaybackState::Playing
                }
            }
        }
    }

    pub fn progress_secs(&self) -> f64 {
        let inner = self.inner.lock().unwrap();
        inner.elapsed_secs()
    }

    pub fn duration_secs(&self) -> f64 {
        let inner = self.inner.lock().unwrap();
        inner.duration
    }

    pub fn sample_rate(&self) -> u32 {
        let inner = self.inner.lock().unwrap();
        inner.sample_rate
    }

    pub fn seek(&self, seconds: f64) -> Result<(), PlaybackError> {
        let mut inner = self.inner.lock().unwrap();
        let duration = inner.duration;
        let offset = if seconds < 0.0 { 0.0 } else { seconds.min(duration) };
        Self::stop_inner(&mut inner);
        self.start_playback_inner(&mut inner, offset)
    }

    pub fn set_volume_db(&self, db: f64) {
        let inner = self.inner.lock().unwrap();
        if let Some(control) = inner.control.as_ref() {
            control.set_volume_db(db);
        }
    }

    pub fn set_filter(&self, profile: FilterProfile) {
        let inner = self.inner.lock().unwrap();
        if let Some(control) = inner.control.as_ref() {
            control.set_filter(profile);
        }
    }

    pub fn sink_empty(&self) -> bool {
        let inner = match self.inner.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };
        match &inner.stream_sink {
            Some((_, sink)) => {
                if !sink.empty() {
                    return false;
                }
                if let Some(play_start) = inner.play_start {
                    let elapsed = play_start.elapsed();
                    let mut net = elapsed - inner.total_paused;
                    if inner.paused {
                        if let Some(pause_start) = inner.pause_start {
                            net -= pause_start.elapsed();
                        }
                    }
                    net > Duration::from_secs(1)
                } else {
                    false
                }
            }
            None => false,
        }
    }

    pub fn advance_queue(&self) {
        let max_retries = 20;
        for _ in 0..max_retries {
            let next_path = match self.advance_queue_internal() {
                Some(p) => p,
                None => {
                    self.stop();
                    return;
                }
            };

            let (sample_rate, duration) = {
                let file = match File::open(&next_path) {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("[Queue] 跳过无法打开的文件 {}: {}", next_path.display(), e);
                        continue;
                    }
                };
                let decoder = match Decoder::new(file) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("[Queue] 跳过无法解码的文件 {}: {}", next_path.display(), e);
                        continue;
                    }
                };
                let sr = decoder.sample_rate();
                let dur = decoder.total_duration().map(|d| d.as_secs_f64()).unwrap_or(0.0);
                (sr, dur)
            };

            let mut inner = match self.inner.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            if inner.paused || inner.stream_sink.is_none() {
                return;
            }
            Self::stop_inner(&mut inner);
            inner.file_path = Some(next_path);
            inner.duration = duration;
            inner.sample_rate = sample_rate;
            if let Err(e) = self.start_playback_inner(&mut inner, 0.0) {
                eprintln!("[Queue] 播放下一首失败: {}", e);
            }
            return;
        }
        self.stop();
    }

    pub fn get_peak(&self) -> f32 {
        let inner = match self.inner.lock() {
            Ok(guard) => guard,
            Err(_) => return 0.0,
        };
        match inner.meter {
            Some(ref meter) => f32::from_bits(meter.load(Ordering::Relaxed)),
            None => 0.0,
        }
    }

    pub fn get_spectrum_bands(&self) -> [f32; 64] {
        let inner = self.inner.lock().unwrap();
        match inner.spectrum_output.as_ref() {
            Some(output) => output.get_bands(),
            None => [0.0f32; 64],
        }
    }

    pub fn get_magnitude(&self) -> f32 {
        let inner = self.inner.lock().unwrap();
        match inner.spectrum_output.as_ref() {
            Some(output) => {
                let bands = output.get_bands();
                let sum: f32 = bands.iter().sum();
                sum / BAND_COUNT as f32
            }
            None => 0.0,
        }
    }

    pub fn set_playback_mode(&self, mode: PlaybackMode) {
        let mut inner = self.inner.lock().unwrap();
        inner.mode = mode;
        if mode == PlaybackMode::Shuffle && !inner.original_queue.is_empty() {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            let mut shuffled = inner.original_queue.clone();
            shuffled.shuffle(&mut rng);
            if let Some(current) = &inner.file_path {
                shuffled.retain(|p| p != current);
                inner.queue = shuffled;
            } else {
                inner.queue = shuffled;
            }
        }
    }

    pub fn playback_mode(&self) -> PlaybackMode {
        let inner = self.inner.lock().unwrap();
        inner.mode
    }

    pub fn set_queue(&self, paths: Vec<PathBuf>) {
        let mut inner = self.inner.lock().unwrap();
        inner.queue = paths.clone();
        inner.original_queue = paths;
        if inner.stream_sink.is_none() {
            inner.queue.clear();
        }
    }

    pub fn enqueue(&self, path: PathBuf) {
        let mut inner = self.inner.lock().unwrap();
        inner.queue.push(path.clone());
        inner.original_queue.push(path);
    }

    pub fn get_queue(&self) -> Vec<PathBuf> {
        let inner = self.inner.lock().unwrap();
        inner.queue.clone()
    }

    pub fn clear_queue(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.queue.clear();
        inner.original_queue.clear();
    }

    pub fn set_playlist(&self, paths: Vec<PathBuf>, current_index: usize) {
        let mut inner = self.inner.lock().unwrap();
        inner.original_queue = paths.clone();
        if current_index + 1 < paths.len() {
            inner.queue = paths[current_index + 1..].to_vec();
        } else {
            inner.queue.clear();
        }
    }

    pub fn set_output_device(&self, device_name: Option<String>) {
        let mut inner = self.inner.lock().unwrap();
        inner.output_device = device_name;
    }

    pub fn output_device(&self) -> Option<String> {
        let inner = self.inner.lock().unwrap();
        inner.output_device.clone()
    }

    pub fn current_file_path(&self) -> Option<PathBuf> {
        let inner = self.inner.lock().unwrap();
        inner.file_path.clone()
    }

    fn advance_queue_internal(&self) -> Option<PathBuf> {
        let mut inner = match self.inner.lock() {
            Ok(g) => g,
            Err(_) => return None,
        };

        if inner.paused || inner.stream_sink.is_none() {
            return None;
        }

        let pop_front = |queue: &mut Vec<PathBuf>| -> Option<PathBuf> {
            if queue.is_empty() { None } else { Some(queue.remove(0)) }
        };

        match inner.mode {
            PlaybackMode::Sequential => pop_front(&mut inner.queue),
            PlaybackMode::LoopOne => inner.file_path.clone(),
            PlaybackMode::LoopAll => {
                if inner.queue.is_empty() {
                    inner.queue = inner.original_queue.clone();
                }
                pop_front(&mut inner.queue)
            }
            PlaybackMode::Shuffle => {
                if inner.queue.is_empty() {
                    use rand::seq::SliceRandom;
                    let mut rng = rand::thread_rng();
                    let mut shuffled = inner.original_queue.clone();
                    if let Some(current) = &inner.file_path {
                        shuffled.retain(|p| p != current);
                    }
                    shuffled.shuffle(&mut rng);
                    inner.queue = shuffled;
                }
                pop_front(&mut inner.queue)
            }
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}
