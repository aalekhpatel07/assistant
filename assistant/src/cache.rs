use redis::AsyncCommands;

#[derive(Debug, Clone)]
pub struct Cache {
    client: redis::Client,
}

const SPEECH_PREFIX: &str = "tts:";
const SPEECH_TO_TEXT_EXPIRY: u64 = 60 * 60 * 24 * 7;

impl Cache {
    pub fn open<I>(info: I) -> Result<Self, redis::RedisError>
    where
        I: redis::IntoConnectionInfo,
    {
        Ok(Self {
            client: redis::Client::open(info.into_connection_info()?)?,
        })
    }

    fn serialize_into_bytes(contents: &[i16]) -> impl Iterator<Item = u8> + '_ {
        contents.iter().flat_map(|v| v.to_le_bytes())
    }
    fn deserialize_from_bytes(contents: &[u8]) -> impl Iterator<Item = i16> + '_ {
        contents
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
    }

    pub async fn store_wav_file(
        &self,
        text: &str,
        contents: &[i16],
    ) -> Result<(), redis::RedisError> {
        let key = format!("{}{}", SPEECH_PREFIX, text).as_bytes().to_owned();
        let mut conn = self.client.get_async_connection().await?;
        let compressed: Vec<u8> = Self::serialize_into_bytes(contents).collect();
        conn.set_ex(key, compressed, SPEECH_TO_TEXT_EXPIRY).await?;

        Ok(())
    }

    pub async fn load_wav_file(&self, text: &str) -> Result<Option<Vec<i16>>, redis::RedisError> {
        let key = format!("{}{}", SPEECH_PREFIX, text).as_bytes().to_owned();
        let mut conn = self.client.get_async_connection().await?;

        let Some(value): Option<Vec<u8>> = conn.get(key).await? else {
            return Ok(None);
        };

        Ok(Some(Self::deserialize_from_bytes(&value).collect()))
    }
}
