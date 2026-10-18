# Development

## Database setup
to setup the database schema, log in to the postgress database manually, and execute the following statements:
```sql
CREATE USER scoretracker_dev;
ALTER USER scoretracker_dev WITH PASSWORD 'password_here';
CREATE SCHEMA IF NOT EXISTS scoretracker_dev AUTHORIZATION scoretracker_dev;
```

then use the following commands to compile and build and set up the project:
```sh
cargo build
cargo run config init
cargo run config set database_connection "postgresql://scoretracker_dev:password_here@localhost"
cargo run config set database_schema "scoretracker_dev"
cargo run server init
cargo run server set database_connection "postgresql://scoretracker_dev:password_here@localhost"
cargo run server set database_schema "scoretracker_dev"
cargo run db init scoretracker_dev
cargo run db import "example/db_import"
npm i
cargo run gen all  # generates `.ts` files from Rust source code
npx tsc --build    # or: `npx tsc --watch` for continuous compilation
```

to delete the schema use the following statement in the sql shell:
```sql
DROP SCHEMA IF EXISTS scoretracker_dev CASCADE;
```


# Contributing

if you ever want to contribute to this for some reason, you are forbidden to use large language models to generate code. use your brain instead please.