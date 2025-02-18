pub use tklog;
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        {
            $crate::mirror::core::tools::logger::tklog::info!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        {
            $crate::mirror::core::tools::logger::tklog::error!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        {
            $crate::mirror::core::tools::logger::tklog::warn!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        {
            $crate::mirror::core::tools::logger::tklog::debug!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        {
            $crate::mirror::core::tools::logger::tklog::trace!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_fatal {
    ($($arg:tt)*) => {
        {
            $crate::mirror::core::tools::logger::tklog::fatal!($($arg)*);
        }
    };
}

// test
#[cfg(test)]
mod tests {
    #[test]
    fn test_logger() {
        log_info!("This is an info message");
        log_error!("This is an error message");
        log_warn!("This is a warn message");
        log_debug!("This is a debug message");
        log_trace!("This is a trace message");
        log_fatal!("This is a fatal message");
    }
}
