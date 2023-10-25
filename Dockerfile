FROM rust:latest

WORKDIR /app

COPY api api
COPY server/Cargo.toml server
COPY .env .

RUN apt update
RUN apt upgrade -y
RUN apt install -y ffmpeg

RUN curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp
RUN chmod a+rx /usr/local/bin/yt-dlp

RUN cargo b -r

CMD cargo r -r