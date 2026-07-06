hotaru_rt_embassy::define_runtime_worker_pool!(
    pub TestRuntime,
    worker_count = 2,
    job_queue_capacity = 3,
);

fn assert_runtime<Rt: hotaru_rt_embassy::__private::hotaru_core::app::runtime::RuntimeSpec>() {}

#[test]
fn generated_runtime_uses_configured_capacities() {
    assert_runtime::<TestRuntime>();
    assert_eq!(TestRuntime::WORKER_COUNT, 2);
    assert_eq!(TestRuntime::JOB_QUEUE_CAPACITY, 3);
}
