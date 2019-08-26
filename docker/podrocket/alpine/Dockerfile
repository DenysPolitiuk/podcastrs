FROM alpine

RUN apk update \
	&& apk add bash

COPY ./podrocket /podrocket

RUN mkdir -p /home/podcastrs && cd /home/podcastrs \
	&& mv /podrocket /home/podcastrs/ \
	&& chmod 777 podrocket

ENTRYPOINT ["/home/podcastrs/podrocket"]
