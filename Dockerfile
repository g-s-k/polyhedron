FROM rust as build
WORKDIR /app
RUN rustup default nightly

COPY . .
RUN cargo build --release

FROM ruby as prod
WORKDIR /poly
RUN apt-get update && apt-get install bash jq -y
RUN gem install twurl

COPY ./post.sh .
COPY --from=build /app/target/release/rand_poly /bin
CMD ["/bin/bash", "post.sh"]
