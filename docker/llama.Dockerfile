FROM ubuntu:latest
WORKDIR /app
RUN apt update && apt install -y python3 python3-pip python3-venv nvidia-cuda-toolkit
RUN python3 -m pip install torch torchvision torchaudio diffusers transformers accelerate scipy safetensors xformers
COPY docker/llama/llama_config.json .
RUN CMAKE_ARGS="-DLLAMA_CUBLAS=on" FORCE_CMAKE=1 pip3 install llama-cpp-python[server]

