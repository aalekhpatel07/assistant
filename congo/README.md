# Congo

Converse with LLMs via your microphone and your speakers!

## Usage

Spin up the docker-compose cluster and when running the Go binary, please provide the following environment variables:

```
COQUI_BASE_URL=http://localhost:9003
REDIS_ADDR=localhost:6380
WHISPER_BASE_URL=http://localhost:5002
LLAMA_PHI_BASE_URL=http://localhost:5001
```

