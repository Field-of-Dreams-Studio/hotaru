hotaru_rt_embassy::define_runtime_worker_pool!(
    pub TestRuntime,
    worker_count = 2,
    job_queue_capacity = 3,
);

hotaru_rt_embassy::define_runtime_worker_pool!(
    pub CustomRawMutexRuntime,
    worker_count = 1,
    job_queue_capacity = 2,
    raw_mutex = embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
);

fn assert_runtime<Rt: hotaru_rt_embassy::__private::hotaru_core::app::runtime::RuntimeSpec>() {}

#[test]
fn generated_runtime_uses_configured_capacities() {
    assert_runtime::<TestRuntime>();
    assert_eq!(TestRuntime::WORKER_COUNT, 2);
    assert_eq!(TestRuntime::JOB_QUEUE_CAPACITY, 3);
}

#[test]
fn generated_runtime_accepts_custom_raw_mutex() {
    assert_runtime::<CustomRawMutexRuntime>();
    assert_eq!(CustomRawMutexRuntime::WORKER_COUNT, 1);
    assert_eq!(CustomRawMutexRuntime::JOB_QUEUE_CAPACITY, 2);
}
