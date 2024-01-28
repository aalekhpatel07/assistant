FROM ghcr.io/coqui-ai/tts
COPY server /root/TTS
ENTRYPOINT ["python3", "TTS/server/server.py"]
CMD ["--use_cuda", "true", "--show_details", "true"]
