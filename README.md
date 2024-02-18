# Master-Server-Lite
A faster, lighter, in memory ElDewrito master server built on Actix. 


## Build
```
docker build . -t masterserver
```

## Start
```
docker run -d  --name edmaster -p 0.0.0.0:80:8080 masterserver
```
