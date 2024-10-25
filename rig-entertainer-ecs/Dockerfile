FROM public.ecr.aws/docker/library/rust:latest as build

RUN apt-get update
RUN apt-get install musl-tools -y
RUN apt-get -y install pkg-config libssl-dev

WORKDIR /rig-entertainer
COPY . /rig-entertainer

# https://stackoverflow.com/questions/31770604/how-to-generate-statically-linked-executables/31778003#31778003
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM scratch as runtime

COPY --from=build /rig-entertainer/target/x86_64-unknown-linux-musl/release/rig-entertainer-ecs /

CMD ["./rig-entertainer-ecs"]