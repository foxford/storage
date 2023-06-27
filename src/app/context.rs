use std::{collections::BTreeMap, env::var, sync::Arc};
use svc_authn::AccountId;
use svc_authz::{
    cache::{create_pool, AuthzCache, RedisCache},
    ClientMap,
};

use crate::app::{
    config::{AppConfig, AudienceSettings},
    util::{read_s3_config, AudienceEstimator, S3Clients},
};

type S3ClientRef = Arc<S3Clients>;

#[derive(Clone)]
pub struct AppContext {
    pub application_id: AccountId,
    pub authz: ClientMap,
    pub aud_estm: Arc<AudienceEstimator>,
    pub s3: S3ClientRef,
    pub audiences_settings: BTreeMap<String, AudienceSettings>,
}

impl AppContext {
    pub fn build(config: AppConfig) -> Self {
        let cache = var("CACHE_ENABLED")
            .ok()
            .and_then(|val| match val.as_ref() {
                "1" => {
                    let url =
                        var("CACHE_URL").unwrap_or_else(|_| panic!("Missing CACHE_URL variable"));

                    let size = var("CACHE_POOL_SIZE")
                        .map(|val| {
                            val.parse::<u32>()
                                .expect("Error converting CACHE_POOL_SIZE variable into u32")
                        })
                        .unwrap_or_else(|_| 5);
                    let idle_size = var("CACHE_POOL_IDLE_SIZE")
                        .map(|val| {
                            val.parse::<u32>()
                                .expect("Error converting CACHE_POOL_IDLE_SIZE variable into u32")
                        })
                        .ok();
                    let timeout = var("CACHE_POOL_TIMEOUT")
                        .map(|val| {
                            val.parse::<u64>()
                                .expect("Error converting CACHE_POOL_TIMEOUT variable into u64")
                        })
                        .unwrap_or_else(|_| 5);
                    let expiration_time = var("CACHE_EXPIRATION_TIME")
                        .map(|val| {
                            val.parse::<u64>()
                                .expect("Error converting CACHE_EXPIRATION_TIME variable into u64")
                        })
                        .unwrap_or_else(|_| 300);

                    Some(Box::new(RedisCache::new(
                        create_pool(&url, size, idle_size, timeout),
                        expiration_time as usize,
                    )) as Box<dyn AuthzCache>)
                }
                _ => None,
            });

        // Resources
        let s3_clients = read_s3_config(&config.backend).expect("Error reading s3 config");

        let s3 = S3ClientRef::new(s3_clients);

        // Authz
        let aud_estm = Arc::new(AudienceEstimator::new(&config.authz));
        let authz = ClientMap::new(&config.id, cache, config.authz.clone(), None)
            .expect("Error converting authz config to clients");

        Self {
            application_id: config.id.clone(),
            authz,
            aud_estm,
            s3,
            audiences_settings: config.audiences_settings,
        }
    }
}
