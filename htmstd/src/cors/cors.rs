use hotaru_core::executable::middleware::AsyncMiddleware;
use hotaru_core::protocol::{Protocol, RequestContext};
use hotaru_http::traits::HTTP;
use hotaru_http::http_value::{HttpMethod, StatusCode};
use hotaru_http::response::response_templates;
use hotaru_trans::middleware;

use super::cors_settings::*;

middleware! {
    /// The CORS middleware
    pub Cors<HTTP> {
        let cors_settings = req
            .runtime()
            .and_then(|rt| rt.get_config::<AppCorsSettings>())
            .unwrap_or_default()
            .merge(
                &req.endpoint()
                    .and_then(|ep| ep.get_params::<AppCorsSettings>())
                    .unwrap_or_default(),
            );
        if req.method() == HttpMethod::OPTIONS && req.meta().get_header("origin").is_some() && req.meta().get_header("access-control-request-method").is_some() {
            let mut response = response_templates::return_status(StatusCode::NO_CONTENT);
            for (key, value) in cors_settings.write_headers(&req.meta().get_header("origin").unwrap_or("".to_string()), true) {
                response.meta.set_attribute(key, value);
            }
            req.response = response;
            return Ok(req);
        }
        let mut req = next(req).await?;
        for (key, value) in cors_settings.write_headers(&req.meta().get_header("origin").unwrap_or("".to_string()), false) {
            req.response.meta.set_attribute(key, value);
        }
        Ok(req)

    }
}
