FROM rust:latest

WORKDIR /app

COPY api api
COPY server server
COPY bot bot
COPY Cargo.toml .
COPY .env .

RUN apt update
RUN apt upgrade -y
RUN apt install -y ffmpeg

RUN curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp
RUN chmod a+rx /usr/local/bin/yt-dlp

RUN cargo b -r --bin server

CMD cargo r -r --bin server