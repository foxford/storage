use app::config::AuthnMap;
use http::header::HeaderValue;
use tower_web::extract::{Context, Error, Extract, Immediate};
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

fn parse_bearer_token(header: &HeaderValue) -> Result<&str, Error> {
    let val: Vec<&str> = header
        .to_str()
        .map_err(|err| Error::invalid_argument(&err))?
        .split(" ")
        .collect();

    match val[..] {
        ["Bearer", val] => Ok(val),
        _ => Err(Error::invalid_argument(&"invalid Bearer token")),
    }
}

fn parse_access_token(header: &HeaderValue, authn: &AuthnMap) -> Result<Subject, Error> {
    use jose::{dangerous_unsafe_decode, decode, Validation};

    let token = parse_bearer_token(&header)?;
    let dirty =
        dangerous_unsafe_decode::<Claims>(token).map_err(|err| Error::invalid_argument(&err))?;

    let config = authn
        .get(&dirty.claims.iss)
        .ok_or_else(|| Error::invalid_argument(&"issuer of an access token is not allowed"))?;

    let mut validation = Validation::new(config.algorithm);
    validation.set_audience(&config.audience);
    validation.validate_exp = dirty.claims.exp.is_some();
    let data = decode::<Claims>(token, config.key.as_ref(), &validation)
        .map_err(|err| Error::invalid_argument(&err))?;

    let sub = Subject {
        id: data.claims.sub,
        audience: data.claims.aud,
    };
    Ok(sub)
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
            None => Immediate::err(Error::missing_argument()),
        }
    }
}
