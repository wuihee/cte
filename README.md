# Combat Training Engine

The goal of this project is to algorithmically calculate rankings for UFC fighters based on an enhanced Elo rating system that considers fight statistics and performance.

## Usage

### Setup

Run `setup.sh` to setup the SQLite database:

```sh
chmod u+x setup.sh
./setup.sh
```

### Sync Fight Data

Sync historical fight data from ESPN and calculate ratings:

```sh
cargo run -- --sync
```

**Note**: This may take a while as it fetches data from 1993-2025.

### View Rankings (TUI)

After syncing, run the TUI to explore fighter rankings:

```sh
cargo run
```

### Controls

#### Rankings View

| Key                 | Action                    |
| ------------------- | ------------------------- |
| `q`                 | Quit                      |
| `Tab`               | Switch to Predictions     |
| `↑`/`↓` or `j`/`k`  | Navigate fighters         |
| `←`/`→` or `h`/`l`  | Change weight class       |
| `Enter`             | View fighter details      |
| `/`                 | Search fighters           |
| `Esc`               | Back / Cancel search      |
| `PageUp`/`PageDown` | Fast scroll               |
| `Home`/`End`        | Jump to top/bottom        |

#### Predictions View

| Key                | Action                     |
| ------------------ | -------------------------- |
| `q`                | Quit                       |
| `Tab`              | Switch to Rankings         |
| `←`/`→` or `h`/`l` | Change event               |
| `r`                | Refresh predictions        |

### Run Tests

```sh
cargo test
```

### Environment Variables

Create a `.env` file with the following environment variables:

```env
# Path or connection string for the SQLite database.
DATABASE_URL=sqlite:data/app.db
```

## Algorithm

This project uses an **Enhanced Elo Rating System** that goes beyond traditional Elo by incorporating fight performance metrics.

### Base Elo Formula

The foundation uses the standard Elo expected score calculation:

```
E_A = 1 / (1 + 10^((R_B - R_A) / 400))
```

### Enhancements

#### 1. Dynamic K-Factor

New fighters have more volatile ratings that stabilize over time:

- **New fighters** (< 15 fights): K = 32 (high volatility)
- **Experienced fighters** (15+ fights): K = 16 (low volatility)
- Scales linearly between these values

#### 2. Finish Bonuses

Dominant victories earn bonus rating points:

| Finish Type | Multiplier |
| ----------- | ---------- |
| KO/TKO      | 1.25x      |
| Submission  | 1.20x      |
| Decision    | 1.00x      |

#### 3. Early Finish Bonus

First-round finishes (< 5 minutes) receive a 1.15x multiplier.

#### 4. Dominance Factor

Performance statistics contribute up to a 1.3x bonus based on:

- **Knockdowns** (30% weight) - Landing knockdowns vs receiving them
- **Significant Strikes** (25% weight) - Strike differential
- **Control Time** (25% weight) - Ground/clinch control
- **Takedowns** (20% weight) - Takedown success rate

### Final Rating Change

```
ΔRating = K * (Actual - Expected) * PerformanceMultiplier

PerformanceMultiplier = FinishBonus * EarlyBonus * (1 + DominanceFactor * 0.3)
```

### Rating Interpretation

| Rating    | Tier                   |
| --------- | ---------------------- |
| 1400+     | Elite (Champion level) |
| 1200-1399 | Contender              |
| 1000-1199 | Average                |
| 800-999   | Below Average          |
| < 800     | Developing             |

### Why Enhanced Elo?

- **HOW you win matters**: A dominant KO earns more than a split decision
- **Experience adjustment**: Prevents wild swings for veterans
- **Performance metrics**: Uses actual fight data, not just outcomes
- **Still simple**: Maintains interpretability of traditional Elo

## Fight Predictions

The app includes predictions for upcoming UFC events based on fighter Elo ratings.

### How Predictions Work

For each upcoming fight, the system:

1. Looks up both fighters' current Elo ratings from the database
2. Calculates win probability using the Elo expected score formula
3. Displays the prediction with confidence indicators

### Prediction Display

- **Green**: Predicted winner (higher win probability)
- **Red**: Predicted loser (lower win probability)
- **Gray ratings**: Fighter not in database (using default 1000 rating)

### Limitations

- Predictions for fighters not in the database use a default rating of 1000
- The system doesn't account for stylistic matchups, injuries, or weight cuts
- Historical data quality affects prediction accuracy

## Resources

- [ESPN WADL](https://sports.core.api.espn.com/v3/application.wadl?detail=true): Describes ESPN's API.
- [Elo Rating System](https://en.wikipedia.org/wiki/Elo_rating_system): Wikipedia article on Elo.
