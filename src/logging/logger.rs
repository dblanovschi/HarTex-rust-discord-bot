use chrono::Local;

use super::log_level::LogLevel;

use crate::system::{
    terminal::Ansi256
};

use crate::utilities::constants::hartex_version;

crate struct Logger;

impl Logger {
    crate fn log_info(message: impl Into<String>) {
        return println!("[HarTex v{}: {}+08:00] [{}{}{}] {}",
                        hartex_version(),
                        Local::now().format("%Y-%m-%d %H:%M:%S"), 
                        Ansi256 {colour: 2}, 
                        LogLevel::Information, 
                        Ansi256::reset(), 
                        message.into()
                    );
    }

    crate fn log_debug(message: impl Into<String>) {
        return println!("[HarTex v{}: {}+08:00] [{}{}{}] {}",
                        hartex_version(),
                        Local::now().format("%Y-%m-%d %H:%M:%S"), 
                        Ansi256 {colour: 33}, LogLevel::Debug, 
                        Ansi256::reset(),
                        message.into()
                    );
    }

    crate fn log_warning(message: impl Into<String>) {
        return println!("[HarTex v{}: {}+08:00] [{}{}{}] {}",
                        hartex_version(),
                        Local::now().format("%Y-%m-%d %H:%M:%S"), 
                        Ansi256 {colour: 226}, 
                        LogLevel::Warning, 
                        Ansi256::reset(), 
                        message.into()
                    );
    }

    crate fn log_error(message: impl Into<String>) {
        return println!("[HarTex v{}: {}+08:00] [{}{}{}] {}",
                        hartex_version(),
                        Local::now().format("%Y-%m-%d %H:%M:%S"), 
                        Ansi256 {colour: 1}, 
                        LogLevel::Error, 
                        Ansi256::reset(), 
                        message.into()
                    );
    }

    crate fn log_verbose(message: impl Into<String>) {
        return println!("[HarTex v{}: {}+08:00] [{}{}{}] {}",
                        hartex_version(),
                        Local::now().format("%Y-%m-%d %H:%M:%S"), 
                        Ansi256 {colour: 240}, 
                        LogLevel::Verbose, 
                        Ansi256::reset(), 
                        message.into()
                    );
    }
}
