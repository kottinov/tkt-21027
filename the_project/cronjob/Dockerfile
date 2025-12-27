FROM alpine:3.19

RUN apk add --no-cache curl

COPY create-wiki-todo.sh /usr/local/bin/create-wiki-todo.sh
RUN chmod +x /usr/local/bin/create-wiki-todo.sh

CMD ["/usr/local/bin/create-wiki-todo.sh"]