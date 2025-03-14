pub mod postgres_test_container;
pub mod selenium_test_container;

pub(crate) fn tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Runtime::new().unwrap()
}
