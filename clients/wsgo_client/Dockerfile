FROM golang:alpine AS builder

RUN apk update && apk add --no-cache git
WORKDIR /app
COPY . .
RUN go get -d -v
RUN go build -o /go/bin/client

FROM golang:alpine 
RUN apk update && apk add --no-cache git bash iproute2 
COPY --from=builder /go/bin/client /go/bin/client
COPY ./entrypoint.sh .
CMD [ "bash","entrypoint.sh" ]