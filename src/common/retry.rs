use std::{future::Future, pin::Pin};

pub async fn retry<'a, F, T, E>(fut: F, retries: usize) -> Result<T, E>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<T, E>> + 'a>>,
{
    for _ in 0..(retries - 1) {
        let result = fut().await;
        if let Ok(r) = result {
            return Ok(r);
        }
    }

    fut().await
}
