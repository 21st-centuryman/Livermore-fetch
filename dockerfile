FROM rust:alpine as build

WORKDIR /app

RUN apk upgrade
RUN apk add --no-cache git musl-dev

RUN git clone https://github.com/21st-centuryman/Livermore-fetch.git .
RUN cargo build --release

FROM alpine:latest

WORKDIR /app

VOLUME /ticker_list
VOLUME /pull_output
VOLUME /process_output

COPY --from=build /app/target/release/Livermore-fetch /usr/local/bin/

RUN echo '0 14 * * 2 Livermore-fetch -P /ticker_list /pull_output' >> /etc/crontabs/root
RUN echo "0 15 * * 2 Livermore-fetch -p /pull_output /process_output" > /etc/crontabs/root

RUN chmod +x /etc/crontabs/root

CMD ["crond", "-f", "-d", "8"]
