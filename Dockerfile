FROM rust as build
WORKDIR /app
RUN rustup default nightly && rustup target add x86_64-unknown-linux-musl

COPY . .
RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM ruby:alpine as prod
WORKDIR /poly
RUN apk --update add bash jq
RUN gem install twurl

COPY ./post.sh .
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/rand_poly /bin
CMD ["/bin/bash", "post.sh"]
