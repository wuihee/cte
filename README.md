# Combat Training Engine

The goal of this project is to algorithmically calculate rankings for UFC fighters based on some elo-like algorithm.

## Usage

Run `setup.sh` to setup sqlite database.

```sh
chmod u+x setup.sh
./setup.sh
```

Run the engine to sync fight data and calculate ratings:

```sh
cargo run
```

Run tests:

```sh
cargo test
```

### Environment Variables

Create a `.env` file with the following environment variables.

```env
# Path or connection string for the SQLite database.
DATABASE_URL=sqlite:data/app.db
```

## Algorithm

## Resources

- [ESPN WADL](https://sports.core.api.espn.com/v3/application.wadl?detail=true): Describes ESPN's API.
