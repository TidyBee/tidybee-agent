FROM ubuntu:18.04

WORKDIR /home/

COPY . .

RUN bash ./startup.sh

ENV PATH="/root/.cargo/bin:$PATH"