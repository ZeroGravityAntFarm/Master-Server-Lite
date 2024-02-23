# Master-Server-Lite
A faster, lighter, in memory ElDewrito master server built on Actix. 


## Build
Get edmaster binary from releases, place in ./target/debug/edmaster or compile your own with 
```cargo run --bin edmaster```

Build container image
```
docker build . -t masterserver
```

## Start
```
docker run -d  --name edmaster -p 0.0.0.0:80:8080 masterserver
```
