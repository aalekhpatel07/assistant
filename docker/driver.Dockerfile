FROM golang:bullseye

ENV DEBIAN_FRONTEND noninteractive
RUN apt-get update -y
RUN apt-get install -y --no-install-recommends alsa-utils libsndfile1-dev libportaudio2 libasound-dev portaudio19-dev
RUN apt-get clean

WORKDIR /app

COPY congo .
RUN go mod download && go mod verify
RUN go build -v -o /usr/local/bin/congo congo.go


CMD ["congo"]
