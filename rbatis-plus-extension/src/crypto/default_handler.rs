//! 默认加密处理器（对标 Java `DefaultEncryptedFieldHandler`）。
//!
//! 使用 XOR + Base64 实现，仅用于演示和测试。
//! 生产环境请实现 `EncryptedFieldHandler` trait 接入 AES/KMS 等强加密。

use super::handler::EncryptedFieldHandler;

/// 默认加密处理器（XOR + Base64，对标 Java `DefaultEncryptedFieldHandler`）。
///
/// **安全警告**：此实现仅用于演示，不提供真实安全性。
/// 生产环境请使用 AES-256-GCM 等强加密算法。
///
/// # 对应 Java
///
/// - `com.baomidou.mybatisplus.enhance.crypto.handler.DefaultEncryptedFieldHandler`
#[derive(Debug, Clone)]
pub struct DefaultEncryptedFieldHandler {
    /// XOR 密钥（默认 "rbatis-plus"）。
    key: Vec<u8>,
}

impl Default for DefaultEncryptedFieldHandler {
    fn default() -> Self {
        Self {
            key: b"rbatis-plus".to_vec(),
        }
    }
}

impl DefaultEncryptedFieldHandler {
    /// 使用自定义密钥创建。
    pub fn new(key: &[u8]) -> Self {
        Self { key: key.to_vec() }
    }

    /// XOR 运算（对称：encrypt 和 decrypt 使用相同逻辑）。
    fn xor_transform(&self, input: &[u8]) -> Vec<u8> {
        input
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ self.key[i % self.key.len()])
            .collect()
    }
}

impl EncryptedFieldHandler for DefaultEncryptedFieldHandler {
    /// XOR 加密 + Base64 编码。
    fn encrypt(&self, value: &str) -> String {
        let xored = self.xor_transform(value.as_bytes());
        base64_encode(&xored)
    }

    /// Base64 解码 + XOR 解密。
    fn decrypt(&self, encrypted: &str) -> String {
        match base64_decode(encrypted) {
            Some(decoded) => {
                let xored = self.xor_transform(&decoded);
                String::from_utf8_lossy(&xored).to_string()
            }
            None => encrypted.to_string(), // 解码失败返回原值
        }
    }

    /// HMAC = SHA256-like（简化版：XOR + hex 编码）。
    fn hmac(&self, value: &str) -> String {
        let xored = self.xor_transform(value.as_bytes());
        hex_encode(&xored)
    }
}

/// 简易 Base64 编码（不依赖外部 crate）。
fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// 简易 Base64 解码。
fn base64_decode(input: &str) -> Option<Vec<u8>> {
    let input: Vec<u8> = input.bytes().filter(|&b| b != b'\n' && b != b'\r').collect();
    if input.len() % 4 != 0 {
        return None;
    }
    let mut result = Vec::new();
    for chunk in input.chunks(4) {
        let a = base64_val(chunk[0])? as u32;
        let b = base64_val(chunk[1])? as u32;
        let c = if chunk[2] == b'=' { 0 } else { base64_val(chunk[2])? as u32 };
        let d = if chunk[3] == b'=' { 0 } else { base64_val(chunk[3])? as u32 };
        let triple = (a << 18) | (b << 12) | (c << 6) | d;
        result.push((triple >> 16) as u8);
        if chunk[2] != b'=' {
            result.push((triple >> 8 & 0xFF) as u8);
        }
        if chunk[3] != b'=' {
            result.push((triple & 0xFF) as u8);
        }
    }
    Some(result)
}

fn base64_val(b: u8) -> Option<u8> {
    match b {
        b'A'..=b'Z' => Some(b - b'A'),
        b'a'..=b'z' => Some(b - b'a' + 26),
        b'0'..=b'9' => Some(b - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        b'=' => Some(0),
        _ => None,
    }
}

/// 十六进制编码。
fn hex_encode(input: &[u8]) -> String {
    input.iter().map(|b| format!("{:02x}", b)).collect()
}
