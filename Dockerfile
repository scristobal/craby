FROM rust:1.64-bullseye as builder
ENV PATH "/root/.cargo/bin:${PATH}"

WORKDIR /app/src

COPY ./ ./
RUN cargo build --release

RUN cp ./target/release/craby /usr/local/bin/

CMD ["craby"]