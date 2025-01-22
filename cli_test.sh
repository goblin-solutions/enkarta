# just reorder the output
# that sorting call is totally googled I am no bash wizard.
cargo run -- ./fixtures/in.csv | tr ' ' '\n' | sort -t',' -k1,1n | diff - ./fixtures/out.csv
