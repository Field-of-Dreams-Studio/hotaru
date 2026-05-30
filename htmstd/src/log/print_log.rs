use hotaru_core::executable::middleware::AsyncMiddleware;
use hotaru_core::protocol::{Protocol, RequestContext};
use hotaru_http::traits::HTTP;
use hotaru_trans::middleware;

middleware! {
    /// Minimal request logger. Prints `<method> <path>` before the handler
    /// runs and `<method> <path> <status>` after. Drop-in replacement for the
    /// 0.7-era `htmstd::PrintLog` re-export.
    pub PrintLog<HTTP> {
        println!("{} {}", req.method(), req.path());
        let mut req = next(req).await?;
        println!(
            "{} {} {}",
            req.method(),
            req.path(),
            req.response.meta.start_line.status_code(),
        );
        Ok(req)
    }
}
