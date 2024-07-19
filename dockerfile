# Use the Alpine Linux base image
FROM rust:alpine

RUN apk upgrade
RUN apk add --no-cache git musl-dev

# Set the working directory to /app
WORKDIR /app

# Copy the Git repository into the container
RUN git clone https://github.com/21st-centuryman/Livermore-fetch.git .
# Build the Rust project
RUN cargo build 
RUN ls /app
# Define environment variables
ARG mode
ARG size

ENV mode=$mode
ENV size=$size
# Define volumes
VOLUME /ticker_list
VOLUME /pull_output
VOLUME /process_output

CMD ["sh", "-c", "cargo run -- -P /ticker_list /pull_output "]
CMD ["sh", "-c", "cargo run -- -p /pull_output /process_output $size"]
