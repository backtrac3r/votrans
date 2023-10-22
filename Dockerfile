FROM rust:latest

RUN apt update
RUN apt upgrade -y
RUN apt install yt-dlp -y

WORKDIR /app
COPY build.rs .
COPY Cargo.toml .
COPY lib ./lib
COPY model ./model
COPY src ./src
COPY .env .

RUN cargo b -r

CMD cargo r -r