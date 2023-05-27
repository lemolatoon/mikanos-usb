FROM debian:stable

WORKDIR /usr/local/src

RUN apt-get update && apt-get install -y --no-install-recommends git-core subversion make cmake clang python ca-certificates wget

ADD build-stdlib.sh /usr/local/src

CMD ["/usr/local/src/build-stdlib.sh"]
