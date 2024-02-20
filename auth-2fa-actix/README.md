- add env variable in .env 
```
DATABASE_URL=postgresql://root:root@localhost:5432/users
JWT_SECRET=anusikh
HASH_SECRET=panda
```

- run `diesel setup` to create db and table
- `make <command-name>` to build, start server

- you can use a qr code generator to generate qr using the totp url that is stored in the db