FROM rust:1.51 as builder
WORKDIR /opt/auth-gatekeeper
ADD ./Cargo.toml ./Cargo.lock /opt/auth-gatekeeper/
RUN cargo fetch --locked && \
	mkdir -p src && \
	echo "fn main() {}" > src/main.rs && \
	cargo build --release

ADD ./src /opt/auth-gatekeeper/src
RUN touch -a -m src/main.rs && cargo build --release

FROM debian:buster-slim
RUN apt-get update && \ 
	apt-get install -y openssl && \
	rm -rf /var/lib/apt/lists/* && \
	ln -s /opt/auth-gatekeeper/auth-gatekeeper /usr/local/bin/auth-gatekeeper
COPY --from=builder /opt/auth-gatekeeper/target/release/auth-gatekeeper /opt/auth-gatekeeper/auth-gatekeeper
CMD ["/opt/auth-gatekeeper/auth-gatekeeper"]
