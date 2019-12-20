# si-web-app

## Environment Options

You can turn authentication on/off by setting up:
```
#.env.local
VUE_APP_AUTHENTICATION=false
```

## Project setup
```
npm install
```

### Compiles and hot-reloads for development
```
npm run serve
```

### Compiles and minifies for production
```
npm run build
```

### Run your tests
```
npm run test
```

### Lints and fixes files
```
npm run lint
```

### Run your unit tests
```
npm run test:unit
```

### Customize configuration
See [Configuration Reference](https://cli.vuejs.org/config/).

## Docker Container
Generate a docker container to run this service with `make docker-build` and run the container with `docker-run`. Alternatively you can generate and run the container with `make docker`.