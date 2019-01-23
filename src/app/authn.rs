use crate::app::config::AuthnMap;
use http::header::HeaderValue;
use http::StatusCode;
use tower_web::extract::{Context, Error as ExtractError, Extract, Immediate};
use tower_web::util::BufStream;
use tower_web::{Error, ErrorBuilder};

#[derive(Debug)]
pub(crate) struct Subject {
    pub(crate) id: String,
    pub(crate) audience: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: String,
    aud: String,
    sub: String,
    exp: Option<u64>,
}

impl<B: BufStream> Extract<B> for Subject {
    type Future = Immediate<Subject>;

    fn extract(context: &Context) -> Self::Future {
        use http::header;

        let authn = context
            .config::<AuthnMap>()
            .expect("missing an authn config");
        match context.request().headers().get(header::AUTHORIZATION) {
            Some(header) => match parse_access_token(&header, &authn) {
                Ok(sub) => Immediate::ok(sub),
                Err(err) => Immediate::err(err),
            },
            None => Immediate::err(error().status(StatusCode::FORBIDDEN).detail("missing access token").build().into()),
        }
    }
}

fn parse_access_token(header: &HeaderValue, authn: &AuthnMap) -> Result<Subject, ExtractError> {
    use jsonwebtoken::{dangerous_unsafe_decode, decode, Validation};

    let token = parse_bearer_token(&header)?;
    let dirty = dangerous_unsafe_decode::<Claims>(token).map_err(|_| {
        ExtractError::from(
            error()
                .detail("invalid claims of the access token")
                .build(),
        )
    })?;

    let issuer = dirty.claims.iss;
    let config = authn.get(&issuer).ok_or_else(|| {
        let detail = format!("issuer = {} of the access token is not allowed", &issuer);
        ExtractError::from(error().detail(&detail).build())
    })?;

    let mut validation = Validation::new(config.algorithm);
    validation.set_audience(&config.audience);
    validation.validate_exp = dirty.claims.exp.is_some();
    let data = decode::<Claims>(token, config.key.as_ref(), &validation).map_err(|_| {
        ExtractError::from(
            error()
                .detail("verification of the access token is failed")
                .build(),
        )
    })?;

    let sub = Subject {
        id: data.claims.sub,
        audience: data.claims.aud,
    };
    Ok(sub)
}

fn parse_bearer_token(header: &HeaderValue) -> Result<&str, ExtractError> {
    let val: Vec<&str> = header
        .to_str()
        .map_err(|_| {
            ExtractError::from(
                error()
                    .detail("invalid characters in the authorization header")
                    .build(),
            )
        })?
        .split(' ')
        .collect();

    match val[..] {
        ["Bearer", ref val] => Ok(val),
        _ => Err(error()
            .detail("unsupported or invalid type of the access token")
            .build()
            .into()),
    }
}

fn error() -> ErrorBuilder {
    Error::builder()
        .kind("authn_error", "Error processing the access token")
        .status(StatusCode::UNAUTHORIZED)
}
