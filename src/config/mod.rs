use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::{Context, Result};

/// Application configuration
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Config {
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_server_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_server_url: Option<String>,
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        // Try to load from current directory first
        let fallback_path = std::env::current_dir()?
            .join("mcd-coupon-config.json");
        
        if fallback_path.exists() {
            match Self::load_from_path(&fallback_path) {
                Ok(config) => return Ok(config),
                Err(_) => {
                    // 静默失败，直接尝试备用路径
                }
            }
        }
        
        // Fall back to primary path
        let primary_path = Self::get_config_path();
        
        if primary_path.exists() {
            match Self::load_from_path(&primary_path) {
                Ok(config) => return Ok(config),
                Err(_) => {
                    // 静默失败，直接尝试备用路径
                }
            }
        }
        
        // Use default if no config files exist
        Ok(Self::default())
    }
    
    /// Helper method to load config from a specific path
    fn load_from_path(path: &std::path::Path) -> Result<Self> {
        let config_str = fs::read_to_string(path)
            .context(format!("无法读取文件: {}", path.display()))?;
        
        serde_json::from_str(&config_str)
            .context(format!("无法解析文件: {}", path.display()))
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path();
        
        // Try to save with the primary path first
        match self.save_to_path(&config_path) {
            Ok(_) => return Ok(()),
            Err(_) => {
                // 静默失败，直接尝试备用路径
            }
        }
        
        // Fall back to current directory
        let fallback_path = std::env::current_dir()?
            .join("mcd-coupon-config.json");
        
        match self.save_to_path(&fallback_path) {
            Ok(_) => {
                // 不在此处打印，让调用者通过app日志显示
                Ok(())
            },
            Err(e) => {
                Err(anyhow::anyhow!("无法保存配置文件到任何位置: {}", e))
            }
        }
    }
    
    /// Helper method to save config to a specific path
    fn save_to_path(&self, path: &std::path::Path) -> Result<()> {
        // Ensure the directory exists
        if let Some(dir) = path.parent() {
            if !dir.exists() {
                fs::create_dir_all(dir)
                    .context(format!("无法创建目录: {}", dir.display()))?;
            }
        }
        
        let config_str = serde_json::to_string_pretty(self)
            .context("无法序列化配置")?;
        
        fs::write(path, config_str)
            .context(format!("无法写入文件: {}", path.display()))
    }
    
    /// Get the path to the configuration file
    pub fn get_config_path() -> std::path::PathBuf {
        // Try to get system config directory first
        if let Some(config_dir) = dirs::config_dir() {
            return config_dir.join("mcd-coupon-tui-rust").join("config.json");
        }
        
        // Fall back to home directory
        let home_dir = dirs::home_dir().expect("无法获取用户主目录");
        home_dir.join(".config").join("mcd-coupon-tui-rust").join("config.json")
    }
    
    /// Check if a valid token exists
    pub fn has_valid_token(&self) -> bool {
        !self.token.trim().is_empty()
    }
}