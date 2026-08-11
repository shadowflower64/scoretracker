# Development

when developing, use cargo to compile the project:
```
cargo build
```

### Web app
if you are also messing with the web app, run this to install dependencies and compile the typescript code:
```
npm i
npx tsc --build
```

for automatic compilation, use:
```
npx tsc --watch
```

### Typescript type generation
to generate typescript code from rust types run:
```
cargo run schema gen
```