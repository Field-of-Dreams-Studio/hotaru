use core::time::Duration;

/// Controls how long a timeout is, or whether it is active at all.
#[derive(Clone, Copy, Debug)]
pub enum TimeoutSetting {
    /// Apply the protocol's built-in default.
    Inherit,
    /// Disable the timeout entirely (needed for long-lived protocols like MQTT).
    Disabled,
    /// Use an explicit duration.
    Fixed(Duration),
}

impl TimeoutSetting {
    /// Convenience constructor: `n` whole seconds.
    #[allow(non_snake_case)]
    pub fn Seconds(n: usize) -> Self {
        Self::Fixed(Duration::from_secs(n as u64))
    }

    /// Convenience constructor: `n` milliseconds.
    #[allow(non_snake_case)]
    pub fn Milliseconds(n: usize) -> Self {
        Self::Fixed(Duration::from_millis(n as u64))
    }
}

/// Shared operational settings used by both server and client runtimes.
pub struct OperationalConfig {
    worker: usize,
    max_connection_time: TimeoutSetting,
    max_frame_process_time: usize,
    connect_timeout: TimeoutSetting,
    request_timeout: TimeoutSetting,
}

impl Default for OperationalConfig {
    fn default() -> Self {
        Self {
            worker: 1,
            max_connection_time: TimeoutSetting::Seconds(30),
            max_frame_process_time: 5,
            connect_timeout: TimeoutSetting::Seconds(30),
            request_timeout: TimeoutSetting::Seconds(30),
        }
    }
}

impl OperationalConfig {
    /// Creates an operational config with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a config from fully specified server and client settings.
    pub fn from_parts(
        worker: usize,
        max_connection_time: TimeoutSetting,
        max_frame_process_time: usize,
        connect_timeout: TimeoutSetting,
        request_timeout: TimeoutSetting,
    ) -> Self {
        Self {
            worker,
            max_connection_time,
            max_frame_process_time,
            connect_timeout,
            request_timeout,
        }
    }

    /// Creates a config while overriding only the server-facing settings.
    pub fn from_server_parts(
        worker: usize,
        max_connection_time: TimeoutSetting,
        max_frame_process_time: usize,
    ) -> Self {
        Self {
            worker,
            max_connection_time,
            max_frame_process_time,
            ..Self::default()
        }
    }

    /// Creates a config while overriding only the client-facing settings.
    pub fn from_client_parts(
        connect_timeout: TimeoutSetting,
        request_timeout: TimeoutSetting,
    ) -> Self {
        Self {
            connect_timeout,
            request_timeout,
            ..Self::default()
        }
    }

    /// Consumes the config and returns all stored parts.
    pub fn into_parts(self) -> (usize, TimeoutSetting, usize, TimeoutSetting, TimeoutSetting) {
        (
            self.worker,
            self.max_connection_time,
            self.max_frame_process_time,
            self.connect_timeout,
            self.request_timeout,
        )
    }

    /// Returns the worker thread count.
    pub fn worker(&self) -> usize {
        self.worker
    }

    /// Returns the maximum connection lifetime setting.
    pub fn max_connection_time(&self) -> TimeoutSetting {
        self.max_connection_time
    }

    /// Returns the maximum frame processing time in seconds.
    pub fn max_frame_process_time(&self) -> usize {
        self.max_frame_process_time
    }

    /// Returns the outbound connect timeout setting.
    pub fn connect_timeout(&self) -> TimeoutSetting {
        self.connect_timeout
    }

    /// Returns the per-request timeout setting.
    pub fn request_timeout(&self) -> TimeoutSetting {
        self.request_timeout
    }

    /// Replaces the worker thread count.
    pub fn set_worker(&mut self, worker: usize) {
        self.worker = worker;
    }

    /// Replaces the maximum connection lifetime setting.
    pub fn set_max_connection_time(&mut self, max_connection_time: TimeoutSetting) {
        self.max_connection_time = max_connection_time;
    }

    /// Replaces the maximum frame processing time in seconds.
    pub fn set_max_frame_process_time(&mut self, max_frame_process_time: usize) {
        self.max_frame_process_time = max_frame_process_time;
    }

    /// Replaces the outbound connect timeout setting.
    pub fn set_connect_timeout(&mut self, connect_timeout: TimeoutSetting) {
        self.connect_timeout = connect_timeout;
    }

    /// Replaces the per-request timeout setting.
    pub fn set_request_timeout(&mut self, request_timeout: TimeoutSetting) {
        self.request_timeout = request_timeout;
    }
}
