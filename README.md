# ZGAF-Master
A faster, lighter, in memory ElDewrito master server built on Actix. 


## Build
```
docker build . -t masterserver
```

## Start
```
docker run -d  --name edmaster -p 127.0.0.1:$port:8080 masterserver
```
