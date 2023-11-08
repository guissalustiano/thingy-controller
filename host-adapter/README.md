# Host adapter server
Listem the RabbitMQ controller topics, update the controller state 
and sync with linux virtual keyboard created by udev

## Architecture notes
I want to the most parallel possible way but keeping separation of concerns.
So I created one task to listem each queue, everthing update one mutex to share 
the new state with another thread who only dispach the keys.
To keep simple I use just one mutex, 
in the end this makes my application as fast as it would be synchronous,
but I learned a lot about mutex and share state between threads, so was worth.

## How to run
```bash
cargo run
```

