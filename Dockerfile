FROM rust as builder
COPY . .
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /target/x86_64-unknown-linux-musl/release/server .
COPY --from=builder /static/ ./static/
EXPOSE 8000
CMD ["./server"]

# works locally for
# docker build . -t <image-name>
# docker run -p 8000:8000 <image-name>

# works on azure app service for
# add app setting (env var) - WEBSITES_PORT=8000
# no need for any startup command