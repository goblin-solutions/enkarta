### Notes:

I use serde and the type system to ensure correctness, but I still test the serialization
code because it's pretty easy to break the expected format by changing the types and those
expecatations should be concrectized somewhere.

Because the events need to be processed chronologically I did not bother making it parallel.
Adding a queue per client that checks ordering of transactions could solve this but would
add uneeded complexity to a cli app that is just going to be running on one hyperthread
in a docker container anyways.

I Opted to validate input rows by hand - throwing an error if any had nonsense values
because the serde structured needed null rows for dispute logic. I opted not to
validate that rows operated on the same precision when running transactions, because
that was not mentioned and seems like a contrived issue.

I used an embedded DB for transactions, assuming there might be hundreds of millions or billions
but opted to keep client info in memory, assuming at max single digit millions. with an entry size of
54 bytes, several million entries would still be under a GB of memory - which is fine for a batch processing job.
Memory constraints were not discussed so I assumed a t2.micro with 1GB of ram.
