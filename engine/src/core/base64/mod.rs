use crate::core::{Example, Function, FunctionEvaluationResult, FunctionInput};
use crate::lang::{lir::Bindings, PatternMeta, Severity};
use crate::package::Package;
use crate::runtime::{ExecutionContext, Output, Pattern, RuntimeError};
use crate::runtime::{PackagePath, World};
use crate::value::RuntimeValue;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::engine::GeneralPurpose;
use base64::Engine;
use std::borrow::Borrow;

use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use crate::runtime::rationale::Rationale;
use serde_json::json;
use std::sync::Arc;

pub fn package() -> Package {
    let mut pkg = Package::new(PackagePath::from_parts(vec!["base64"]))
        .with_documentation(r#"Functionality for processing base64 encoded data"#.to_string());
    pkg.register_function("base64".into(), Base64::new(Alphabet::Standard));
    pkg.register_function("base64-url".into(), Base64::new(Alphabet::UrlNoPad));
    pkg.register_function("base64-encode".into(), Base64Encode);
    pkg
}

const DOCUMENTATION_BASE64: &str = include_str!("base64.adoc");
const DOCUMENTATION_BASE64_ENCODE: &str = include_str!("base64-encode.adoc");
const DOCUMENTATION_BASE64URL: &str = include_str!("base64-url.adoc");

#[derive(Debug)]
enum Alphabet {
    Standard,
    UrlNoPad,
}

impl Alphabet {
    pub fn decoder(&self) -> GeneralPurpose {
        match self {
            Alphabet::Standard => STANDARD,
            Alphabet::UrlNoPad => URL_SAFE_NO_PAD,
        }
    }
}

#[derive(Debug)]
pub struct Base64 {
    alphabet: Alphabet,
}

impl Base64 {
    fn new(alphabet: Alphabet) -> Self {
        Self { alphabet }
    }
}

impl Function for Base64 {
    fn input(&self, _bindings: &[Arc<Pattern>]) -> FunctionInput {
        FunctionInput::String
    }

    fn metadata(&self) -> PatternMeta {
        PatternMeta {
            documentation: match self.alphabet {
                Alphabet::Standard => DOCUMENTATION_BASE64.into(),
                Alphabet::UrlNoPad => DOCUMENTATION_BASE64URL.into(),
            },
            ..Default::default()
        }
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                name: "default".to_string(),
                summary: Some("Simple base64 encoded value".to_string()),
                description: Some(
                    "Validates the base64 string and transforms it into the BLOB.".to_string(),
                ),
                value: json!("SGVsbG8gUm9kbmV5IQ=="),
            },
            Example {
                name: "failure".to_string(),
                summary: Some("Non-base64 encoded value".to_string()),
                description: Some(
                    "Fails to validate, as it is not a base64 encoded value".to_string(),
                ),
                value: json!("foo bar"),
            },
        ]
    }

    fn call<'v>(
        &'v self,
        input: Arc<RuntimeValue>,
        _ctx: ExecutionContext<'v>,
        _bindings: &'v Bindings,
        _world: &'v World,
    ) -> Pin<Box<dyn Future<Output = Result<FunctionEvaluationResult, RuntimeError>> + 'v>> {
        Box::pin(async move {
            let input = (*input).borrow();
            if let Some(inner) = input.try_get_str() {
                let result = self.alphabet.decoder().decode(inner);

                if let Ok(decoded) = result {
                    Ok(Output::Transform(Arc::new(decoded.into())).into())
                } else {
                    Ok((
                        Severity::Error,
                        Rationale::InvalidArgument("Invalid base64 encoding".into()),
                    )
                        .into())
                }
            } else {
                Ok((
                    Severity::Error,
                    Rationale::InvalidArgument("Expected a string".into()),
                )
                    .into())
            }
        })
    }
}

#[derive(Debug)]
pub struct Base64Encode;

impl Function for Base64Encode {
    fn metadata(&self) -> PatternMeta {
        PatternMeta {
            documentation: DOCUMENTATION_BASE64_ENCODE.into(),
            ..Default::default()
        }
    }

    fn call<'v>(
        &'v self,
        input: Arc<RuntimeValue>,
        _ctx: ExecutionContext<'v>,
        _bindings: &'v Bindings,
        _world: &'v World,
    ) -> Pin<Box<dyn Future<Output = Result<FunctionEvaluationResult, RuntimeError>> + 'v>> {
        Box::pin(async move {
            let input = (*input).borrow();
            if let Some(inner) = input.try_get_octets() {
                let result = STANDARD.encode(inner);

                Ok(Output::Transform(Arc::new(result.into())).into())
            } else {
                Ok(Severity::Error.into())
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::runtime::testutil::test_common;
    use crate::{assert_not_satisfied, assert_satisfied};

    #[tokio::test]
    async fn test_base64_invalid() {
        let result = test_common(
            r#"
pattern test = base64::base64
"#,
            "Clearly not base64",
        )
        .await;

        assert_not_satisfied!(result);
    }

    #[tokio::test]
    async fn test_base64_valid() {
        let result = test_common(
            r#"
pattern test = base64::base64
"#,
            "SGVsbG8gV29ybGQh",
        )
        .await;

        assert_satisfied!(&result);
        assert_eq!(result.output(), Arc::new(b"Hello World!".into()));
    }

    #[tokio::test]
    async fn test_base64_encode_valid() {
        let result = test_common(
            r#"
pattern test = base64::base64-encode
"#,
            b"Hello World!",
        )
        .await;

        assert_satisfied!(&result);
        assert_eq!(result.output(), Arc::new("SGVsbG8gV29ybGQh".into()));
    }
}
