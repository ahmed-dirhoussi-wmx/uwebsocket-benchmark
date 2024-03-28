FROM node:20-bullseye
RUN apt-get update && \
    apt-get install iproute2 iputils-ping -y

WORKDIR /app
COPY src /app/src
COPY package.json .yarn yarn.lock entrypoint_server.sh /app/
RUN yarn install

EXPOSE 9001

CMD [ "bash", "entrypoint_server.sh" ]