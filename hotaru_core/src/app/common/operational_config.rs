/// Shared operational settings used by both server and client runtimes.
pub struct OperationalConfig {
    binding_address: String,
    worker: usize,
    max_connection_time: usize,
    max_frame_process_time: usize,
    connect_timeout: usize,
    request_timeout: usize,
}

impl Default for OperationalConfig {
    fn default() -> Self {
        Self {
            binding_address: String::from("127.0.0.1:3003"),
            worker: 1,
            max_connection_time: 30,
            max_frame_process_time: 5,
            connect_timeout: 30,
            request_timeout: 30,
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
        binding_address: String,
        worker: usize,
        max_connection_time: usize,
        max_frame_process_time: usize,
        connect_timeout: usize,
        request_timeout: usize,
    ) -> Self {
        Self {
            binding_address,
            worker,
            max_connection_time,
            max_frame_process_time,
            connect_timeout,
            request_timeout,
        }
    }

    /// Creates a config while overriding only the server-facing settings.
    pub fn from_server_parts(
        binding_address: String,
        worker: usize,
        max_connection_time: usize,
        max_frame_process_time: usize,
    ) -> Self {
        Self {
            binding_address,
            worker,
            max_connection_time,
            max_frame_process_time,
            ..Self::default()
        }
    }

    /// Creates a config while overriding only the client-facing settings.
    pub fn from_client_parts(connect_timeout: usize, request_timeout: usize) -> Self {
        Self {
            connect_timeout,
            request_timeout,
            ..Self::default()
        }
    }

    /// Consumes the config and returns all stored parts.
    pub fn into_parts(self) -> (String, usize, usize, usize, usize, usize) {
        (
            self.binding_address,
            self.worker,
            self.max_connection_time,
            self.max_frame_process_time,
            self.connect_timeout,
            self.request_timeout,
        )
    }

    /// Returns the binding address.
    pub fn binding_address(&self) -> &str {
        &self.binding_address
    }

    /// Returns the worker thread count.
    pub fn worker(&self) -> usize {
        self.worker
    }

    /// Returns the maximum connection lifetime in seconds.
    pub fn max_connection_time(&self) -> usize {
        self.max_connection_time
    }

    /// Returns the maximum frame processing time in seconds.
    pub fn max_frame_process_time(&self) -> usize {
        self.max_frame_process_time
    }

    /// Returns the connect timeout in seconds.
    pub fn connect_timeout(&self) -> usize {
        self.connect_timeout
    }

    /// Returns the request timeout in seconds.
    pub fn request_timeout(&self) -> usize {
        self.request_timeout
    }

    /// Replaces the binding address.
    pub fn set_binding<T: Into<String>>(&mut self, binding_address: T) {
        self.binding_address = binding_address.into();
    }

    /// Replaces the worker thread count.
    pub fn set_worker(&mut self, worker: usize) {
        self.worker = worker;
    }

    /// Replaces the maximum connection lifetime in seconds.
    pub fn set_max_connection_time(&mut self, max_connection_time: usize) {
        self.max_connection_time = max_connection_time;
    }

    /// Replaces the maximum frame processing time in seconds.
    pub fn set_max_frame_process_time(&mut self, max_frame_process_time: usize) {
        self.max_frame_process_time = max_frame_process_time;
    }

    /// Replaces the connect timeout in seconds.
    pub fn set_connect_timeout(&mut self, connect_timeout: usize) {
        self.connect_timeout = connect_timeout;
    }

    /// Replaces the request timeout in seconds.
    pub fn set_request_timeout(&mut self, request_timeout: usize) {
        self.request_timeout = request_timeout;
    }
}
