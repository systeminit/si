FROM node:10.16.0

WORKDIR /app
COPY . /app
RUN make deps-install

CMD make run