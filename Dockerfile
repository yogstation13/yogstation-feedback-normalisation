FROM rust:1.40 as build

COPY ./ ./

RUN cargo build --release

FROM ubuntu:18.04

ENV DEBIAN_FRONTEND=noninteractive

COPY --from=build /target/release/feedback-normalization /

CMD /feedback-normalization