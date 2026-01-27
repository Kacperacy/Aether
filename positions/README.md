# Test Positions for Algorithm Comparison

This directory contains test positions from [Stockfish Books](https://github.com/official-stockfish/books) organized by game phase.

## Directory Structure

```
positions/
├── opening/          # Positions after 3-4 moves from start (game_phase > 200)
├── middlegame/       # Complex middlegame positions (game_phase 80-200)
└── endgame/          # Endgame positions (game_phase < 80)
```

## Downloading Positions

Run the download script:

```bash
./scripts/download_positions.sh
```

This will download the following files from Stockfish Books:

| Phase | File | Positions | Description |
|-------|------|-----------|-------------|
| Opening | `noob_3moves.epd` | ~100k | Positions after 3 moves |
| Middlegame | `UHO_XXL_+0.90_+1.19.epd` | Large | Balanced middlegame |
| Endgame | `endgames.epd` | 157,846 | Various endgames |

## Using with fastchess

```bash
# Opening test
fastchess \
  -engine cmd=./target/release/aether name=FullAlphaBeta \
  -engine cmd=./target/release/aether-mcts name=MCTS \
  -openings file=positions/opening/noob_3moves.epd format=epd order=random \
  -each tc=10+0.1 \
  -rounds 100

# Endgame test
fastchess \
  -engine cmd=./target/release/aether name=FullAlphaBeta \
  -engine cmd=./target/release/aether-mcts name=MCTS \
  -openings file=positions/endgame/endgames.epd format=epd order=random \
  -each tc=30+0.3 \
  -rounds 100
```

## Using with built-in benchfile command

```bash
./target/release/aether
> benchfile positions/opening/noob_3moves.epd 10
```

Output shows statistics grouped by game phase (opening/middlegame/endgame).

## Source

All positions are from the [Stockfish Books Repository](https://github.com/official-stockfish/books)
and are licensed under CC0-1.0 (public domain).
