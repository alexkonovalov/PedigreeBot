FROM rust:1.57

COPY . .

EXPOSE ${PORT}

RUN apt update && \
    apt install -y software-properties-common && \
    apt-get install -y graphviz && \
    cargo build --release

CMD ["./target/release/pedigree-bot"]


