use axum::async_trait;
use std::cmp::Ordering;

use crate::extract::user_lang::UserLanguageSource;

/// A [`UserLanguageSource`] that reads languages from the `Accept-Language` header.
/// 
/// This source may return multiple languages. Languages are returned in order of their
/// quality values.
/// 
/// # Example
/// 
/// ```rust
/// # use axum::{Router, extract::Extension, routing::get};
/// # use axum_extra::extract::user_lang::{UserLanguage, AcceptLanguageSource};
/// #
/// let source = AcceptLanguageSource;
///
/// let app = Router::new()
///    .route("/home", get(handler))
///    .layer(
///        Extension(
///            UserLanguage::config()
///                .add_source(source)
///                .build(),
///    ));
///
/// # let _: Router = app;  
/// # async fn handler() {}
/// ```
#[derive(Debug, Clone)]
pub struct AcceptLanguageSource;

#[async_trait]
impl UserLanguageSource for AcceptLanguageSource {
    async fn languages_from_parts(&self, parts: &mut http::request::Parts) -> Vec<String> {
        let Some(accept_language) = parts.headers.get("Accept-Language") else {
            return vec![];
        };

        let Ok(accept_language) = accept_language.to_str() else {
            return vec![];
        };

        parse_quality_values(accept_language)
            .into_iter()
            .map(|(lang, _)| lang.to_string())
            .collect()
    }
}

/// Parse quality values from the `Accept-Language` header.
fn parse_quality_values(values: &str) -> Vec<(&str, f32)> {
    let mut values = values.split(',');
    let mut quality_values = Vec::new();

    while let Some(value) = values.next() {
        let mut value = value.trim().split(';');
        let (value, quality) = (value.next(), value.next());

        let Some(value) = value else {
            // empty quality value entry
            continue;
        };

        let quality = if let Some(quality) = quality.and_then(|q| q.strip_prefix("q=")) {
            quality.parse::<f32>().unwrap_or(0.0)
        } else {
            1.0
        };

        quality_values.push((value, quality));
    }

    quality_values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    quality_values
}
