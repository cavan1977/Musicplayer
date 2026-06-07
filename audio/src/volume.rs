//! 数字音量控制 - 64-bit 浮点高精度
//! 音量步进 0.01 dB，工作在线性域
//! 放在 FIR 滤波器之后，输出到声卡之前

/// 音量控制器，支持 0.01dB 步进
#[derive(Debug, Clone)]
pub struct VolumeControl {
    /// 当前增益 (线性值)，1.0 = 0dB
    gain: f64,
    /// 最大增益 (0dB)
    max_gain: f64,
    /// 最小增益 (对应 -120dB 防削波)
    min_gain: f64,
     /// 每步 dB 变化量
     db_step: f64,
}

impl VolumeControl {
    /// 创建新的音量控制器
    /// 
    /// # Arguments
    /// - `initial_db`: 初始音量，单位 dB (0.0 = unity)
    /// - `dB_step`: 步长，默认 0.01 dB
    /// - `range_db`: 动态范围，默认 -120dB 到 0dB
     pub fn new(initial_db: f64, db_step: f64, range_db: f64) -> Self {
        let min_gain = 10.0_f64.powf((initial_db - range_db) / 20.0);
        let max_gain = 10.0_f64.powf(initial_db / 20.0);
        
         Self {
             gain: 10.0_f64.powf(initial_db / 20.0),
             max_gain,
             min_gain,
             db_step,
         }
    }
    
    /// 默认音量控制器：0 dB，0.01 dB 步长，范围 -120 dB
    pub fn default() -> Self {
        Self::new(0.0, 0.01, 120.0)
    }
    
    /// 设置音量 (dB)
    /// 自动限制在动态范围内
    pub fn set_volume_db(&mut self, db: f64) {
        let min_db = 20.0 * self.min_gain.log10();
        let max_db = 20.0 * self.max_gain.log10();
        let clamped = db.max(min_db).min(max_db);
        self.gain = 10.0_f64.powf(clamped / 20.0);
    }
    
    /// 增加音量 (dB 步进)
    pub fn volume_up(&mut self) {
        let current_db = 20.0 * self.gain.log10();
         self.set_volume_db(current_db + self.db_step);
    }
    
    /// 降低音量 (dB 步进)
    pub fn volume_down(&mut self) {
        let current_db = 20.0 * self.gain.log10();
         self.set_volume_db(current_db - self.db_step);
    }
    
    /// 获取当前线性增益
    pub fn gain(&self) -> f64 {
        self.gain
    }
    
    /// 获取当前 dB 值
    pub fn volume_db(&self) -> f64 {
        20.0 * self.gain.log10()
    }
    
    /// 应用到交错 PCM 数据 (f64)
    /// 输入/输出均为交错格式 [L,R,L,R,...]
    pub fn apply_interleaved(&self, input: &[f64]) -> Vec<f64> {
        if input.is_empty() {
            return Vec::new();
        }
        input.iter().map(|&sample| sample * self.gain).collect()
    }
    
    /// 应用到平面 PCM 数据 (每个声道数组)
    pub fn apply_planar(&self, channels: &[&[f64]]) -> Vec<f64> {
        let mut out = Vec::new();
        for ch in channels {
            out.extend(ch.iter().map(|&s| s * self.gain));
        }
        out
    }
    
    /// 纯增益乘数 (不分配 Vec，用于流式处理)
    pub fn multiply_gain(&self, samples: &mut [f64]) {
        for s in samples.iter_mut() {
            *s *= self.gain;
        }
    }

    /// 纯增益乘数 f32 版本 (不分配 Vec，用于流式处理)
    pub fn multiply_gain_f32(&self, samples: &mut [f32]) {
        let gain_f32 = self.gain as f32;
        for s in samples.iter_mut() {
            *s *= gain_f32;
        }
    }
    
    /// 静音 (增益设为 0)
    pub fn mute(&mut self) {
        self.gain = 0.0;
    }
    
    /// 检查是否静音
    pub fn is_muted(&self) -> bool {
        self.gain == 0.0
    }
    
    /// 重置为默认
    pub fn reset(&mut self) {
        self.gain = self.max_gain;
    }
}
