FROM rust:latest

WORKDIR /app

COPY server/lib lib
COPY model model
COPY server/src src
COPY server/build.rs .
COPY server/Cargo.toml .
COPY .env .

RUN apt update
RUN apt upgrade -y
RUN apt install -y ffmpeg

RUN curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp
RUN chmod a+rx /usr/local/bin/yt-dlp
RUN yt-dlp -U

RUN cargo b -r

CMD cargo r -r