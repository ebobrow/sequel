# SQL clone

Run server with
```sh
cargo run --bin sequel-server
```
Run client with
```sh
cargo run --bin sequel-client
```
They connect on port 3000. The client provides a command prompt where you can
enter commands `SELECT [column names or *] FROM [table name]`, `INSERT INTO
[table name] (column name,*) VALUES (val,*)`, and `CREATE TABLE
[table name] ([column name] [string/number] [optional constraints],*)`.
