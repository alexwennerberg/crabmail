 rm -rf site \
   && cargo build \
   && ./target/debug/crabmail -m $1 \
   && cp templates/static/style.css site/ && cp templates/static/style.css site/threads/
