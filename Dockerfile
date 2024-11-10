FROM rustlang/rust:nightly AS prereq
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-chef -y
RUN cargo binstall cargo-leptos -y 
RUN rustup target add wasm32-unknown-unknown
RUN curl -sL https://deb.nodesource.com/setup_20.x | bash 
RUN apt-get update && apt-get install nodejs
RUN npm install -g sass


FROM prereq AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM prereq as cacher
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --bin=manga-tracker --target-dir=target/    --recipe-path recipe.json
RUN cargo chef cook --release --target-dir=target/front --target=wasm32-unknown-unknown    --recipe-path recipe.json

FROM prereq as builder
COPY . /app
WORKDIR /app

COPY --from=cacher /app/target /app/target
COPY --from=cacher /usr/local/cargo /usr/local/cargo


RUN cargo leptos build --release


FROM rustlang/rust:nightly as runner

WORKDIR /app

COPY --from=builder /app/target/release/manga-tracker /app/
COPY --from=builder /app/target/site /app/site
COPY --from=builder /app/migrations /app/migrations

ENV OUTPUT_NAME "manga-tracker"
ENV LEPTOS_OUTPUT_NAME "manga-tracker"
ENV LEPTOS_SITE_ROOT "site"
ENV LEPTOS_SITE_PKG_DIR "pkg"
ENV LEPTOS_ASSETS_DIR "assets"
ENV LEPTOS_SITE_ADDR "0.0.0.0:4000"
ENV APP_ENV "prod" 

EXPOSE 4000


CMD ["./manga-tracker"]
