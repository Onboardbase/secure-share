use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Secret {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecretResponse {
    pub status: String
}

impl Secret {
    pub fn secrets_from_string(secrets: Vec<String>) -> Vec<Secret> {
        secrets
            .into_iter()
            .filter_map(|secret| Secret::secret_from_string(secret).ok())
            .collect::<Vec<_>>()
    }

    fn secret_from_string(secret: String) -> Result<Secret> {
        let key_value_pair = secret.split(',').collect::<Vec<_>>();
        if key_value_pair.len() > 2 {
            return Err(anyhow!("Key Value Pair for secret is incompatible"));
        }
        if key_value_pair.len() < 2 {
            return Err(anyhow!("Key or Value not found for {secret}"));
        }
        let secret = Secret {
            key: key_value_pair.first().unwrap().to_string(),
            value: key_value_pair.get(1).unwrap().to_string(),
        };
        Ok(secret)
    }
}
