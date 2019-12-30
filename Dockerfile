FROM rust:1.40 as build

COPY ./ ./

RUN cargo build --release

FROM ubuntu:18.04

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get -y install ca-certificates libssl-dev

COPY --from=build /target/release/feedback-normalization /

CMD /feedback-normalization