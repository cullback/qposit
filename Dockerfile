# Stage 1: Building the application
FROM rust:1.76.0 as builder

# Create a new empty shell project
RUN USER=root cargo new --bin qposit
WORKDIR /qposit

# Copy our manifests
COPY ./Cargo.toml ./Cargo.toml
COPY ./lobster ./lobster

# This trick will cache our dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy our source code
COPY ./src ./src
COPY ./lobster ./lobster
COPY ./static ./static
COPY ./db/migrations ./db/migrations
COPY ./templates ./templates

# Set environment variables required for build
ENV DATABASE_URL=sqlite:db/db.db

# we need the database to exist in order to build the application
RUN cargo install sqlx-cli
RUN sqlx database create
RUN sqlx migrate run --source db/migrations
RUN rm ./target/release/deps/qposit*
RUN cargo build --release

# Stage 2: Preparing the final image
FROM debian:bookworm-slim

COPY --from=builder /qposit/target/release/qposit .

CMD ["./qposit"]
