FROM node:26.2.0-alpine AS builder
WORKDIR /app

COPY ./package.json ./package-lock.json ./
COPY ./frontend/package.json ./frontend/package.json
RUN npm ci

COPY ./frontend ./frontend
ENV PUBLIC_WEBSOCKET_URL="ws://localhost:8080/ws/mrsroni"
RUN npm run build

FROM nginx:1.31.1-alpine AS runner
COPY ./deploy/docker/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder --chown=nginx:nginx /app/frontend/build /usr/share/nginx/html