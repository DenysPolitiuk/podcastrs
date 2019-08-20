from ubuntu

RUN apt-get update && apt-get -y upgrade && apt-get install -y wget \
	&& apt-get install -y libssl1.0.0:amd64 \
	&& apt-get -y autoremove && apt-get -y clean

RUN mkdir -p /home/podcastrs && cd /home/podcastrs \
	&& wget https://podcastrs-deploy.s3.ca-central-1.amazonaws.com/podrocket \
	&& chmod +x podrocket

RUN groupadd -r podcastrs && useradd -r -g podcastrs podcastrs \
	&& chown -R podcastrs:podcastrs /home/podcastrs

USER podcastrs

CMD ["/home/podcastrs/podrocket"] 
