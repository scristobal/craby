FROM rust:1.65-bullseye as builder
ENV PATH "/root/.cargo/bin:${PATH}"

WORKDIR /app/src

COPY ./ ./
RUN cargo build --release


FROM rust:1.65.0-slim-bullseye

COPY --from=builder /app/src/target/release/craby /usr/local/bin/

CMD ["craby"]