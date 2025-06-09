FROM rust:buster AS builder
LABEL maintainer="schizo99@gmail.com"

WORKDIR /build
COPY ./parser .

RUN RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-unknown-linux-gnu

FROM node:22-alpine AS web-builder
WORKDIR /app
COPY web/package*.json .
RUN npm ci
COPY web/ .
RUN npm run build
RUN npm prune --production

FROM node:22-alpine
WORKDIR /app
COPY --from=builder /build/target/x86_64-unknown-linux-gnu/release//ttyrec-parser /usr/local/bin/ttyrec-parser
COPY --from=web-builder /app/build .
COPY --from=web-builder /app/node_modules node_modules/
COPY web/package.json .
ENV NODE_ENV=production

EXPOSE 3000
ENTRYPOINT ["node", "/app/index.js"]