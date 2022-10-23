FROM rust:1.64-bullseye as builder
ENV PATH "/root/.cargo/bin:${PATH}"

WORKDIR /app/src

COPY ./ ./
RUN cargo build --release

CMD ["./target/release/craby"]