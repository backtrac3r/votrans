FROM rust:latest

WORKDIR /app

COPY server/lib lib
COPY server/model model
COPY server/src src
COPY server/build.rs .
COPY server/Cargo.toml .
COPY .env .

RUN apt update
RUN apt upgrade -y
RUN apt install yt-dlp -y
RUN cargo b -r

CMD cargo r -r