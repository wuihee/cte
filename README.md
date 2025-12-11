# Combat Training Engine

In light of ranking controversies, the goal of this project is to algorithmically calculate rankings for UFC fighters based on some elo-like algorithm.

## Usage

```sh
cargo run
```

### Environment Variables

Create a `.env` file with the following environment variables.

```env
# Path or connection string for the SQLite database.
DATABASE_URL=sqlite:data/app.db
```

## Resources

- [ESPN WADL](https://sports.core.api.espn.com/v3/application.wadl?detail=true): Describes ESPN's API.
