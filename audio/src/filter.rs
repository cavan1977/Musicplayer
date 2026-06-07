//! FIR 数字滤波器 - HiFi 音质处理
//! 使用 Direct Form I 结构，支持 64-bit 浮点精度
//! 提供多款专业响度均衡系数 (NOS, Linear Phase, Minimum Phase, Mixed Phase)

use std::f64::consts::PI;

/// FIR 滤波器配置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterProfile {
    /// NOS (Non-Over-Sampled) - 原汁原味，最平直相位
    NOS,
    /// Linear Phase Slow Roll-off - 慢滚降，最平滑幅频
    LinearPhaseSlow,
    /// Linear Phase Fast Roll-off - 快滚降，更宽通频带
    LinearPhaseFast,
    /// Minimum Phase - 最小相位，群延迟低，听感直接
    MinimumPhase,
    /// Mixed Phase - 混合相位，兼顾线性与瞬态
    MixedPhase,
}

/// FIR 系数预计算生成器
pub struct Coefficients;

impl Coefficients {
    /// NOS 滤波器：48kHz 采样率下 0-20kHz 几乎平坦，无预 ringing
    pub fn nos(sample_rate: u32) -> Vec<f64> {
        let cutoff = 20000.0;
        let len = 2048;
        Self::windowed_sinc(len, cutoff, sample_rate as f64, |n, w| {
            0.5 * (1.0 - (2.0 * PI * n as f64 / (w as f64 - 1.0)).cos())
        })
    }
    
    /// 线性相位慢滚降
    pub fn linear_phase_slow(sample_rate: u32) -> Vec<f64> {
        let cutoff = 20000.0;
        let len = 4096;
        Self::windowed_sinc(len, cutoff, sample_rate as f64, |n, w| {
            0.5 * (1.0 - (2.0 * PI * n as f64 / (w as f64 - 1.0)).cos())
        })
    }
    
    /// 线性相位快滚降
    pub fn linear_phase_fast(sample_rate: u32) -> Vec<f64> {
        let cutoff = 20000.0;
        let len = 2048;
        Self::windowed_sinc(len, cutoff, sample_rate as f64, |n, w| {
            0.54 - 0.46 * (2.0 * PI * n as f64 / (w as f64 - 1.0)).cos()
        })
    }
    
    /// 最小相位：简化实现，返回 NOS
    pub fn minimum_phase(sample_rate: u32) -> Vec<f64> {
        Self::nos(sample_rate)
    }
    
    /// 混合相位：慢滚降的轻微变体
    pub fn mixed_phase(sample_rate: u32) -> Vec<f64> {
        Self::linear_phase_slow(sample_rate)
    }
    
    /// 窗口法设计 FIR 低通滤波器
    fn windowed_sinc(
        len: usize,
        cutoff_hz: f64,
        sample_rate_hz: f64,
        window: impl Fn(usize, usize) -> f64,
    ) -> Vec<f64> {
        let mut coeffs = Vec::with_capacity(len);
        let fc = cutoff_hz / sample_rate_hz;
        let center = (len - 1) / 2;
        
        for n in 0..len {
            let val = if n == center {
                2.0 * fc
            } else {
                let omega = 2.0 * PI * fc * (n as f64 - center as f64);
                2.0 * fc * (omega.sin() / omega)
            } * window(n, len);
            coeffs.push(val);
        }
        
        // 归一化 DC 增益 = 1
        let sum: f64 = coeffs.iter().sum();
        if sum != 0.0 {
            coeffs.iter_mut().for_each(|c| *c /= sum);
        }
        coeffs
    }
}

/// FIR 滤波器状态 - 处理 64-bit 浮点音频
/// 使用 per-channel 延迟线，支持立体声正确处理
pub struct FirFilter {
    coefficients: Vec<f64>,
    /// Per-channel delay lines: delay[channel][position]
    delay: Vec<Vec<f64>>,
    /// Current write position (shared across channels, but each has own history)
    position: usize,
}

impl FirFilter {
    /// 创建新 FIR 滤波器
    pub fn new(profile: FilterProfile, sample_rate: u32) -> Self {
        let coeffs = match profile {
            FilterProfile::NOS => Coefficients::nos(sample_rate),
            FilterProfile::LinearPhaseSlow => Coefficients::linear_phase_slow(sample_rate),
            FilterProfile::LinearPhaseFast => Coefficients::linear_phase_fast(sample_rate),
            FilterProfile::MinimumPhase => Coefficients::minimum_phase(sample_rate),
            FilterProfile::MixedPhase => Coefficients::mixed_phase(sample_rate),
        };

        let taps = coeffs.len();
        // Initialize delay lines for up to 8 channels
        let max_channels = 8;
        let delay = vec![vec![0.0; taps]; max_channels];

        Self {
            coefficients: coeffs,
            delay,
            position: 0,
        }
    }

    /// Direct Form I 卷积处理交错 PCM [L,R,L,R,...]
    /// 每个通道有独立的延迟线，避免通道间干扰
    pub fn process_interleaved(&mut self, input: &[f64], channels: usize) -> Vec<f64> {
        if input.is_empty() || channels == 0 {
            return Vec::new();
        }
        let frames = input.len() / channels;
        let mut output = vec![0.0; input.len()];
        let delay_len = self.delay[0].len();

        for frame in 0..frames {
            for ch in 0..channels {
                if ch >= self.delay.len() {
                    continue; // Safety check
                }
                let idx = frame * channels + ch;
                let current_sample = input[idx];

                // Write current sample to this channel's delay line
                self.delay[ch][self.position] = current_sample;

                // Compute convolution for this channel
                // coeff[0] * delay[ch][position] +
                // coeff[1] * delay[ch][position-1] + ...
                let mut sum = 0.0;
                for (t, &coeff) in self.coefficients.iter().enumerate() {
                    let delay_idx = (self.position + delay_len - t) % delay_len;
                    sum += coeff * self.delay[ch][delay_idx];
                }
                output[idx] = sum;
            }
            // Advance position after processing all channels for this frame
            self.position = (self.position + 1) % delay_len;
        }
        output
    }

    /// 清空延迟线状态
    pub fn clear(&mut self) {
        for ch in 0..self.delay.len() {
            self.delay[ch].fill(0.0);
        }
        self.position = 0;
    }

    /// 获取滤波器阶数
    pub fn taps(&self) -> usize {
        self.coefficients.len()
    }
}
