use app::config::AuthnMap;
use http::header::HeaderValue;
use tower_web::error::{Error, ErrorKind};
use tower_web::extract::{Context, Error as ExtractError, Extract, Immediate};
use tower_web::util::BufStream;

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
            // This error won't be thrown if subject is used as an optional argument
            // NOTE: anonymous access is forbidden
            None => Immediate::err(authz_error()),
        }
    }
}

fn parse_bearer_token(header: &HeaderValue) -> Result<&str, ExtractError> {
    let val: Vec<&str> = header
        .to_str()
        // NOTE: invalid characters in authorization header
        .map_err(|_| authn_error())?
        .split(' ')
        .collect();

    // NOTE: unsupported or invalid type of the access token
    match val[..] {
        ["Bearer", val] => Ok(val),
        _ => Err(authn_error()),
    }
}

fn parse_access_token(header: &HeaderValue, authn: &AuthnMap) -> Result<Subject, ExtractError> {
    use jose::{dangerous_unsafe_decode, decode, Validation};

    let token = parse_bearer_token(&header)?;
    let dirty =
        // NOTE: invalid access token – {err}
        dangerous_unsafe_decode::<Claims>(token).map_err(|_err| authn_error())?;

    let config = authn
        .get(&dirty.claims.iss)
        // NOTE: issuer of the access token is not allowed
        .ok_or_else(|| authn_error())?;

    let mut validation = Validation::new(config.algorithm);
    validation.set_audience(&config.audience);
    validation.validate_exp = dirty.claims.exp.is_some();
    let data = decode::<Claims>(token, config.key.as_ref(), &validation)
        // NOTE: invalid access token – {err}
        .map_err(|_err| authn_error())?;

    let sub = Subject {
        id: data.claims.sub,
        audience: data.claims.aud,
    };
    Ok(sub)
}

fn authn_error() -> ExtractError {
    ExtractError::web(Error::from(ErrorKind::unauthorized()))
}

fn authz_error() -> ExtractError {
    ExtractError::web(Error::from(ErrorKind::forbidden()))
}
