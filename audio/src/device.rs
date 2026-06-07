//! 音频输出设备枚举与选择

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::Device as CpalDevice;

/// 枚举所有可用的音频输出设备名称
/// 在 Windows 上使用 WASAPI host 以确保蓝牙设备也能被枚举
pub fn list_output_devices() -> Vec<String> {
    let mut devices: Vec<String> = Vec::new();

    #[cfg(target_os = "windows")]
    {
        if let Ok(wasapi_host) = cpal::host_from_id(cpal::HostId::Wasapi) {
            if let Ok(output_devices) = wasapi_host.output_devices() {
                for device in output_devices {
                    if let Ok(name) = device.name() {
                        if !devices.contains(&name) {
                            devices.push(name);
                        }
                    }
                }
            }
        }
    }

    if devices.is_empty() {
        let host = cpal::default_host();
        if let Ok(output_devices) = host.output_devices() {
            devices = output_devices.filter_map(|d| d.name().ok()).collect();
        }
    }

    devices
}

/// 根据名称查找 cpal 设备
/// 先尝试 WASAPI host，再尝试默认 host
pub fn find_device_by_name(name: &str) -> Option<CpalDevice> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(wasapi_host) = cpal::host_from_id(cpal::HostId::Wasapi) {
            if let Ok(output_devices) = wasapi_host.output_devices() {
                for device in output_devices {
                    if let Ok(n) = device.name() {
                        if n == name {
                            return Some(device);
                        }
                    }
                }
            }
        }
    }

    let host = cpal::default_host();
    match host.output_devices() {
        Ok(output_devices) => {
            for device in output_devices {
                if let Ok(n) = device.name() {
                    if n == name {
                        return Some(device);
                    }
                }
            }
            None
        }
        Err(_) => None,
    }
}
