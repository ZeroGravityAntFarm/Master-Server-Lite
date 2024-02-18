FROM ubuntu:24.04

WORKDIR /bin

ADD ./target/debug/edmaster /bin

RUN chmod +x edmaster

ENTRYPOINT ["edmaster"]
