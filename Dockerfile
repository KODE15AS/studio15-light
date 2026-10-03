# Plassholder-container: serverer kun en statusside til arbeidsflaten bygges.
# Basisimage pinnet (lærdom fra Studio 15: aldri :latest).
FROM busybox:1.37.0

RUN adduser -D -H web
WORKDIR /srv
COPY index.html .
USER web
EXPOSE 8100
CMD ["httpd", "-f", "-p", "8100", "-h", "/srv"]
