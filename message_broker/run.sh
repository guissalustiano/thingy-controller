#!/bin/sh

podman run -it --rm \
    --name rabbitmq \
    -e RABBITMQ_DEFAULT_USER=user \
    -e RABBITMQ_DEFAULT_PASS=password \
    -p 5672:5672 -p 15672:15672 \
    docker.io/rabbitmq:3-management
