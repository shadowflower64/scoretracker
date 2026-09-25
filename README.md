# Development

when developing, use cargo to compile the project:
```sh
cargo build
```

### Web app
if you are also messing with the web app, run this to install dependencies and compile the typescript code:
```sh
npm i
npx tsc --build
```

for automatic compilation, use:
```sh
npx tsc --watch
```

### Typescript type generation
to generate typescript code from rust types run:
```sh
cargo run gen all
```

# Database setup

to setup the database schema, log in to the postgress database manually, and execute the following statements:
```sql
CREATE USER scoretracker_dev;
ALTER USER scoretracker_dev WITH PASSWORD 'password_here';
CREATE SCHEMA IF NOT EXISTS scoretracker_dev AUTHORIZATION scoretracker_dev;
```

this will create a new user `scoretracker_dev`, and an empty schema named `scoretracker_dev` with the correct permissions. set the password to whatever you like, and then store it in `server.toml` or `toolkit.toml`. and then to actually create the tables inside the schema run the command:
```sh
cargo run db init scoretracker_dev
```

the `scoretracker_dev` above is the name of the schema to initialize, so you can actually change the schema name to whatever you like. the name of the user should be stored in `server.toml` or `toolkit.toml` along with the password, so you can also name the user however you like.

to delete the schema use the following statement:
```sql
DROP SCHEMA IF EXISTS scoretracker_dev CASCADE;
```


# Contributing

if you ever want to contribute to this for some reason, you are forbidden to use large language models to generate code. use your brain instead please.