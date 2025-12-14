FROM apache/superset:4.1.0

ENV SUPERSET_LOAD_EXAMPLES=no
ENV SUPERSET_SECRET_KEY=supersecretkey
EXPOSE 8088

CMD ["superset", "run", "-p", "8088", "--with-threads", "--reload", "--debugger"]
