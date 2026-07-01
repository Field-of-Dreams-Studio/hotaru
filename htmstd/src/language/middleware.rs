//! The `Accept-Language` parsing middleware.

use hotaru_core::executable::middleware::AsyncMiddleware;
use hotaru_core::protocol::{Protocol, RequestContext};
use hotaru_http::traits::HTTP;
use hotaru_trans::middleware;

use super::settings::PreferredLanguageSettings;
use super::types::PreferredLanguage;

middleware! {
    /// Parses `Accept-Language` and stores [`PreferredLanguage`] in `req.params`.
    pub PreferredLanguageMiddleware<HTTP> {
        let language_settings = req
            .runtime()
            .and_then(|rt| rt.get_config::<PreferredLanguageSettings>())
            .unwrap_or_default()
            .merge(
                &req.endpoint()
                    .and_then(|ep| ep.get_params::<PreferredLanguageSettings>())
                    .unwrap_or_default(),
            );

        let accept_language = req.meta().get_header("accept-language");
        let preferred_language = PreferredLanguage::from_accept_language(
            accept_language.as_deref(),
            language_settings.fallback_language(),
        );
        req.params.set(preferred_language);

        next(req).await
    }
}
