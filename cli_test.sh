# just reorder the output
cargo run -- ./fixtures/in.csv | tr ' ' '\n' | sort -t',' -k1,1n | diff - ./fixtures/out.csv
