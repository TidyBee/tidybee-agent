# tidybee-hub

## Build
### Using Docker
This project can be build and run using the following command:
```shell
docker build -t tidybee-hub .
```

### Using your machine
Build and run the project with these two commands:
```shell
dotnet publish -c Release -o out && cd out && \
dotnet tidybee-hub.dll
```
