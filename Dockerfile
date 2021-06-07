FROM ekidd/rust-musl-builder:stable as builder
RUN USER=root cargo new --bin fiatprices
WORKDIR /home/rust/src/fiatprices
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs
ADD src ./src/
RUN rm ./target/x86_64-unknown-linux-musl/release/deps/fiatprices*
RUN cargo build --release

FROM alpine:latest
EXPOSE 8080
ENV TZ=Etc/UTC \
    APP_USER=appuser \
    RUST_LOG=sqlx=warn,tide=info,ureq=warn,info
RUN addgroup -S $APP_USER && adduser -S -g $APP_USER $APP_USER
COPY --from=builder /home/rust/src/fiatprices/target/x86_64-unknown-linux-musl/release/fiatprices /usr/src/app/fiatprices
RUN chown -R $APP_USER:$APP_USER /usr/src/app
USER $APP_USER
WORKDIR /usr/src/app
ENTRYPOINT /app/fiatprices --server=1 --index=0