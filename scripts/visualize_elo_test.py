#!/usr/bin/env python3
"""
Aether Chess Engine - Tournament Visualization for Master's Thesis
Comparative analysis of decision tree search algorithms.

Generates charts for comparing:
- Alpha-Beta, NegaScout, MTD(f), MCTS algorithms
- Head-to-head tournament results
- Win/Draw/Loss statistics
"""

import re
import sys
import io
from pathlib import Path
from collections import defaultdict
from dataclasses import dataclass

import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import numpy as np

# Fix Windows console encoding for Polish characters
if sys.platform == 'win32':
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8', errors='replace')

# Configuration for thesis-quality plots
plt.rcParams['font.size'] = 11
plt.rcParams['axes.titlesize'] = 13
plt.rcParams['axes.labelsize'] = 12
plt.rcParams['figure.dpi'] = 150

# Polish labels for thesis
POLISH_LABELS = {
    'wins': 'Wygrane',
    'draws': 'Remisy',
    'losses': 'Przegrane',
    'score': 'Wynik',
    'games': 'Partie',
    'algorithm': 'Algorytm',
    'win_rate': 'Procent wygranych',
    'score_percent': 'Wynik procentowy',
    'opponent': 'Przeciwnik',
    'performance': 'Wydajność',
    'avg_game_length': 'Średnia długość partii',
    'ply': 'Półruchy',
}

# Algorithm display names (matching PGN player names)
ALGORITHM_NAMES = {
    'FullAlphaBeta': 'Alpha-Beta (pełny)',
    'PureAlphaBeta': 'Alpha-Beta (czysty)',
    'NegaScout': 'NegaScout',
    'MTDf': 'MTD(f)',
    'MCTS': 'MCTS',
    'ClassicMCTS': 'MCTS (klasyczny)',
}

# Colors for algorithms - consistent across all charts
ALGORITHM_COLORS = {
    'FullAlphaBeta': '#2ecc71',
    'PureAlphaBeta': '#27ae60',
    'NegaScout': '#3498db',
    'MTDf': '#9b59b6',
    'MCTS': '#e74c3c',
    'ClassicMCTS': '#c0392b',
}


def get_algorithm_color(algo):
    """Get color for algorithm, with fallback."""
    return ALGORITHM_COLORS.get(algo, '#7f8c8d')


def get_algorithm_label(algo):
    """Get Polish label for algorithm."""
    return ALGORITHM_NAMES.get(algo, algo)


@dataclass
class GameResult:
    white: str
    black: str
    result: str  # "1-0", "0-1", "1/2-1/2"
    termination: str = ""
    ply_count: int = 0
    time_control: str = ""
    opening_fen: str = ""


@dataclass
class PlayerStats:
    name: str
    wins: int = 0
    losses: int = 0
    draws: int = 0
    total_plies: int = 0
    games_with_plies: int = 0

    @property
    def games(self) -> int:
        return self.wins + self.losses + self.draws

    @property
    def score(self) -> float:
        return self.wins + 0.5 * self.draws

    @property
    def score_percent(self) -> float:
        if self.games == 0:
            return 0.0
        return (self.score / self.games) * 100

    @property
    def win_percent(self) -> float:
        if self.games == 0:
            return 0.0
        return (self.wins / self.games) * 100

    @property
    def avg_game_length(self) -> float:
        if self.games_with_plies == 0:
            return 0.0
        return self.total_plies / self.games_with_plies


def parse_pgn(pgn_path: Path) -> list[GameResult]:
    """Parse PGN file and extract game results."""
    games = []

    with open(pgn_path, 'r', encoding='utf-8', errors='ignore') as f:
        content = f.read()

    # Split into individual games
    game_blocks = re.split(r'\n\n(?=\[Event)', content)

    for block in game_blocks:
        if not block.strip():
            continue

        white_match = re.search(r'\[White "([^"]+)"\]', block)
        black_match = re.search(r'\[Black "([^"]+)"\]', block)
        result_match = re.search(r'\[Result "([^"]+)"\]', block)
        termination_match = re.search(r'\[Termination "([^"]+)"\]', block)
        ply_match = re.search(r'\[PlyCount "(\d+)"\]', block)
        tc_match = re.search(r'\[TimeControl "([^"]+)"\]', block)
        fen_match = re.search(r'\[FEN "([^"]+)"\]', block)

        if white_match and black_match and result_match:
            games.append(GameResult(
                white=white_match.group(1),
                black=black_match.group(1),
                result=result_match.group(1),
                termination=termination_match.group(1) if termination_match else "",
                ply_count=int(ply_match.group(1)) if ply_match else 0,
                time_control=tc_match.group(1) if tc_match else "",
                opening_fen=fen_match.group(1) if fen_match else ""
            ))

    return games


def get_all_players(games: list[GameResult]) -> list[str]:
    """Extract all player/algorithm names."""
    players = set()
    for game in games:
        players.add(game.white)
        players.add(game.black)
    return sorted(players)


def calculate_head_to_head(games: list[GameResult], players: list[str]) -> dict:
    """Calculate head-to-head results matrix."""
    results = {p1: {p2: {'wins': 0, 'draws': 0, 'losses': 0}
                    for p2 in players} for p1 in players}

    for game in games:
        if game.white not in players or game.black not in players:
            continue

        if game.result == "1-0":
            results[game.white][game.black]['wins'] += 1
            results[game.black][game.white]['losses'] += 1
        elif game.result == "0-1":
            results[game.white][game.black]['losses'] += 1
            results[game.black][game.white]['wins'] += 1
        elif game.result == "1/2-1/2":
            results[game.white][game.black]['draws'] += 1
            results[game.black][game.white]['draws'] += 1

    return results


def calculate_overall_stats(games: list[GameResult], players: list[str]) -> dict[str, PlayerStats]:
    """Calculate overall statistics for each player."""
    stats = {p: PlayerStats(name=p) for p in players}

    for game in games:
        if game.white in stats:
            if game.result == "1-0":
                stats[game.white].wins += 1
            elif game.result == "0-1":
                stats[game.white].losses += 1
            else:
                stats[game.white].draws += 1

            if game.ply_count > 0:
                stats[game.white].total_plies += game.ply_count
                stats[game.white].games_with_plies += 1

        if game.black in stats:
            if game.result == "0-1":
                stats[game.black].wins += 1
            elif game.result == "1-0":
                stats[game.black].losses += 1
            else:
                stats[game.black].draws += 1

            if game.ply_count > 0:
                stats[game.black].total_plies += game.ply_count
                stats[game.black].games_with_plies += 1

    return stats


# ============================================================================
# VISUALIZATION FUNCTIONS
# ============================================================================

def plot_algorithm_comparison_bar(stats: dict[str, PlayerStats], output_dir: Path):
    """Bar chart comparing all algorithms by wins/draws/losses."""
    fig, ax = plt.subplots(figsize=(12, 7))

    algorithms = sorted(stats.keys(), key=lambda x: stats[x].score_percent, reverse=True)
    x = np.arange(len(algorithms))
    width = 0.25

    wins = [stats[a].wins for a in algorithms]
    draws = [stats[a].draws for a in algorithms]
    losses = [stats[a].losses for a in algorithms]

    bars1 = ax.bar(x - width, wins, width, label=POLISH_LABELS['wins'], color='#2ecc71')
    bars2 = ax.bar(x, draws, width, label=POLISH_LABELS['draws'], color='#f39c12')
    bars3 = ax.bar(x + width, losses, width, label=POLISH_LABELS['losses'], color='#e74c3c')

    ax.set_xlabel(POLISH_LABELS['algorithm'], fontsize=12)
    ax.set_ylabel(POLISH_LABELS['games'], fontsize=12)
    ax.set_title('Porównanie algorytmów przeszukiwania - wyniki turniejowe', fontsize=14, fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels([get_algorithm_label(a) for a in algorithms], rotation=15, ha='right')
    ax.legend(loc='upper right')

    # Add value labels
    for bars in [bars1, bars2, bars3]:
        for bar in bars:
            height = bar.get_height()
            if height > 0:
                ax.annotate(f'{int(height)}',
                           xy=(bar.get_x() + bar.get_width()/2, height),
                           xytext=(0, 3), textcoords="offset points",
                           ha='center', va='bottom', fontsize=9)

    plt.tight_layout()
    plt.savefig(output_dir / 'algorithm_comparison.png', dpi=150)
    plt.savefig(output_dir / 'algorithm_comparison.pdf')
    plt.close()


def plot_score_percentage_horizontal(stats: dict[str, PlayerStats], output_dir: Path):
    """Horizontal bar chart showing score percentage for each algorithm."""
    fig, ax = plt.subplots(figsize=(10, 6))

    algorithms = sorted(stats.keys(), key=lambda x: stats[x].score_percent)
    scores = [stats[a].score_percent for a in algorithms]
    labels = [get_algorithm_label(a) for a in algorithms]
    colors = [get_algorithm_color(a) for a in algorithms]

    bars = ax.barh(labels, scores, color=colors)

    # Add 50% reference line
    ax.axvline(x=50, color='gray', linestyle='--', linewidth=1.5, alpha=0.7)
    ax.text(51, -0.7, '50%', fontsize=10, color='gray')

    ax.set_xlabel(POLISH_LABELS['score_percent'] + ' (%)', fontsize=12)
    ax.set_title('Wynik procentowy algorytmów w turnieju', fontsize=14, fontweight='bold')
    ax.set_xlim(0, 100)

    # Add value labels
    for bar, score in zip(bars, scores):
        ax.text(score + 1, bar.get_y() + bar.get_height()/2,
                f'{score:.1f}%', va='center', fontsize=10, fontweight='bold')

    plt.tight_layout()
    plt.savefig(output_dir / 'score_percentage.png', dpi=150)
    plt.savefig(output_dir / 'score_percentage.pdf')
    plt.close()


def plot_head_to_head_heatmap(games: list[GameResult], algorithms: list[str], output_dir: Path):
    """Heatmap showing head-to-head score percentage between algorithms."""
    h2h = calculate_head_to_head(games, algorithms)

    n = len(algorithms)
    matrix = np.zeros((n, n))

    for i, p1 in enumerate(algorithms):
        for j, p2 in enumerate(algorithms):
            if i == j:
                matrix[i][j] = 50  # Self vs self
            else:
                r = h2h[p1][p2]
                total = r['wins'] + r['draws'] + r['losses']
                if total > 0:
                    matrix[i][j] = (r['wins'] + 0.5 * r['draws']) / total * 100
                else:
                    matrix[i][j] = 50

    fig, ax = plt.subplots(figsize=(10, 8))

    im = ax.imshow(matrix, cmap='RdYlGn', vmin=0, vmax=100)

    # Add colorbar
    cbar = ax.figure.colorbar(im, ax=ax)
    cbar.ax.set_ylabel(POLISH_LABELS['score_percent'] + ' (%)', rotation=-90, va="bottom", fontsize=11)

    # Set ticks
    labels = [get_algorithm_label(a) for a in algorithms]
    ax.set_xticks(np.arange(n))
    ax.set_yticks(np.arange(n))
    ax.set_xticklabels(labels, rotation=45, ha='right')
    ax.set_yticklabels(labels)

    # Add text annotations
    for i in range(n):
        for j in range(n):
            if i != j:
                text = ax.text(j, i, f'{matrix[i, j]:.0f}%',
                              ha='center', va='center', fontsize=10,
                              color='white' if matrix[i, j] < 30 or matrix[i, j] > 70 else 'black')

    ax.set_title('Wyniki bezpośrednich pojedynków między algorytmami\n(wynik z perspektywy algorytmu w wierszu)',
                 fontsize=13, fontweight='bold')
    ax.set_xlabel('Przeciwnik', fontsize=12)
    ax.set_ylabel('Algorytm', fontsize=12)

    plt.tight_layout()
    plt.savefig(output_dir / 'head_to_head_heatmap.png', dpi=150)
    plt.savefig(output_dir / 'head_to_head_heatmap.pdf')
    plt.close()


def plot_stacked_results(stats: dict[str, PlayerStats], output_dir: Path):
    """Stacked horizontal bar chart showing W/D/L proportions."""
    fig, ax = plt.subplots(figsize=(12, 6))

    algorithms = sorted(stats.keys(), key=lambda x: stats[x].score_percent, reverse=True)
    labels = [get_algorithm_label(a) for a in algorithms]

    wins = [stats[a].wins for a in algorithms]
    draws = [stats[a].draws for a in algorithms]
    losses = [stats[a].losses for a in algorithms]

    y_pos = np.arange(len(algorithms))

    ax.barh(y_pos, wins, color='#2ecc71', label=POLISH_LABELS['wins'])
    ax.barh(y_pos, draws, left=wins, color='#f39c12', label=POLISH_LABELS['draws'])
    ax.barh(y_pos, losses, left=[w+d for w,d in zip(wins, draws)], color='#e74c3c', label=POLISH_LABELS['losses'])

    ax.set_yticks(y_pos)
    ax.set_yticklabels(labels)
    ax.set_xlabel(POLISH_LABELS['games'], fontsize=12)
    ax.set_title('Rozkład wyników algorytmów (W/R/P)', fontsize=14, fontweight='bold')
    ax.legend(loc='lower right')

    # Add total games annotation
    for i, algo in enumerate(algorithms):
        total = stats[algo].games
        ax.text(total + 0.5, i, f'{total}', va='center', fontsize=10)

    plt.tight_layout()
    plt.savefig(output_dir / 'stacked_results.png', dpi=150)
    plt.savefig(output_dir / 'stacked_results.pdf')
    plt.close()


def plot_win_draw_loss_pie(stats: dict[str, PlayerStats], output_dir: Path):
    """Pie charts for each algorithm showing W/D/L distribution."""
    algorithms = sorted(stats.keys(), key=lambda x: stats[x].score_percent, reverse=True)
    n_algos = len(algorithms)

    cols = 3
    rows = (n_algos + cols - 1) // cols

    fig, axes = plt.subplots(rows, cols, figsize=(4*cols, 4*rows))
    axes = axes.flatten() if n_algos > 1 else [axes]

    colors = ['#2ecc71', '#f39c12', '#e74c3c']
    labels_pie = [POLISH_LABELS['wins'], POLISH_LABELS['draws'], POLISH_LABELS['losses']]

    for idx, algo in enumerate(algorithms):
        ax = axes[idx]
        s = stats[algo]
        sizes = [s.wins, s.draws, s.losses]

        if sum(sizes) > 0:
            wedges, texts, autotexts = ax.pie(sizes, colors=colors, autopct='%1.0f%%',
                                               startangle=90, pctdistance=0.75)
            for autotext in autotexts:
                autotext.set_fontsize(9)
                autotext.set_fontweight('bold')

        ax.set_title(f'{get_algorithm_label(algo)}\n({s.score:.1f}/{s.games} pkt)',
                    fontsize=11, fontweight='bold')

    # Hide empty subplots
    for idx in range(len(algorithms), len(axes)):
        axes[idx].axis('off')

    # Add legend
    fig.legend(labels_pie, loc='lower center', ncol=3, fontsize=10,
               bbox_to_anchor=(0.5, 0.02))

    plt.suptitle('Rozkład wyników dla każdego algorytmu', fontsize=14, fontweight='bold', y=0.98)
    plt.tight_layout(rect=[0, 0.05, 1, 0.95])
    plt.savefig(output_dir / 'wdl_pie_charts.png', dpi=150)
    plt.savefig(output_dir / 'wdl_pie_charts.pdf')
    plt.close()


def plot_tournament_crosstable(games: list[GameResult], all_players: list[str], output_dir: Path):
    """Generate tournament cross-table as a figure."""
    h2h = calculate_head_to_head(games, all_players)
    stats = calculate_overall_stats(games, all_players)

    # Sort by score
    sorted_players = sorted(all_players, key=lambda x: stats[x].score, reverse=True)

    n = len(sorted_players)
    fig, ax = plt.subplots(figsize=(max(12, n * 1.5), max(6, n * 0.8)))
    ax.axis('off')

    # Create table data
    headers = ['#', 'Algorytm', 'Pkt', '%'] + [str(i+1) for i in range(n)]
    table_data = []

    for i, p in enumerate(sorted_players):
        row = [
            str(i + 1),
            get_algorithm_label(p),
            f'{stats[p].score:.1f}/{stats[p].games}',
            f'{stats[p].score_percent:.1f}%'
        ]
        for j, opp in enumerate(sorted_players):
            if i == j:
                row.append('X')
            else:
                r = h2h[p][opp]
                total = r['wins'] + r['draws'] + r['losses']
                if total > 0:
                    score = r['wins'] + 0.5 * r['draws']
                    row.append(f'{score:.1f}/{total}')
                else:
                    row.append('-')
        table_data.append(row)

    table = ax.table(cellText=table_data, colLabels=headers,
                     cellLoc='center', loc='center',
                     colColours=['#e8e8e8'] * len(headers))

    table.auto_set_font_size(False)
    table.set_fontsize(9)
    table.scale(1.2, 1.5)

    # Style header
    for j in range(len(headers)):
        table[(0, j)].set_text_props(fontweight='bold')

    ax.set_title('Tabela turniejowa (cross-table)', fontsize=14, fontweight='bold', pad=20)

    plt.tight_layout()
    plt.savefig(output_dir / 'tournament_crosstable.png', dpi=150, bbox_inches='tight')
    plt.savefig(output_dir / 'tournament_crosstable.pdf', bbox_inches='tight')
    plt.close()


def plot_algorithm_ranking(stats: dict[str, PlayerStats], output_dir: Path):
    """Combined ranking chart with score and win percentage."""
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 6))

    # Sort by score percentage
    algorithms = sorted(stats.keys(), key=lambda x: stats[x].score_percent, reverse=True)

    # Left plot: Score percentage ranking
    scores = [stats[a].score_percent for a in algorithms]
    labels = [get_algorithm_label(a) for a in algorithms]
    colors = [get_algorithm_color(a) for a in algorithms]

    y_pos = np.arange(len(algorithms))
    bars1 = ax1.barh(y_pos, scores, color=colors)
    ax1.set_yticks(y_pos)
    ax1.set_yticklabels(labels)
    ax1.invert_yaxis()
    ax1.set_xlabel(POLISH_LABELS['score_percent'] + ' (%)', fontsize=12)
    ax1.set_title('Ranking wg wyniku procentowego', fontsize=13, fontweight='bold')
    ax1.axvline(x=50, color='gray', linestyle='--', linewidth=1.5, alpha=0.7)

    for bar, score in zip(bars1, scores):
        ax1.text(score + 1, bar.get_y() + bar.get_height()/2,
                f'{score:.1f}%', va='center', fontsize=11, fontweight='bold')

    # Right plot: Win percentage ranking
    win_pcts = [stats[a].win_percent for a in algorithms]

    bars2 = ax2.barh(y_pos, win_pcts, color=colors)
    ax2.set_yticks(y_pos)
    ax2.set_yticklabels(labels)
    ax2.invert_yaxis()
    ax2.set_xlabel(POLISH_LABELS['win_rate'] + ' (%)', fontsize=12)
    ax2.set_title('Ranking wg procentu wygranych', fontsize=13, fontweight='bold')

    for bar, pct in zip(bars2, win_pcts):
        ax2.text(pct + 1, bar.get_y() + bar.get_height()/2,
                f'{pct:.1f}%', va='center', fontsize=11, fontweight='bold')

    plt.suptitle('Porównanie rankingowe algorytmów przeszukiwania',
                 fontsize=14, fontweight='bold', y=1.02)
    plt.tight_layout()
    plt.savefig(output_dir / 'algorithm_ranking.png', dpi=150, bbox_inches='tight')
    plt.savefig(output_dir / 'algorithm_ranking.pdf', bbox_inches='tight')
    plt.close()


def plot_game_length_comparison(stats: dict[str, PlayerStats], output_dir: Path):
    """Compare average game length for each algorithm."""
    algorithms = [a for a in stats.keys() if stats[a].avg_game_length > 0]
    if not algorithms:
        return

    fig, ax = plt.subplots(figsize=(10, 6))

    algorithms = sorted(algorithms, key=lambda x: stats[x].avg_game_length)
    lengths = [stats[a].avg_game_length for a in algorithms]
    labels = [get_algorithm_label(a) for a in algorithms]
    colors = [get_algorithm_color(a) for a in algorithms]

    bars = ax.barh(labels, lengths, color=colors)

    ax.set_xlabel(POLISH_LABELS['avg_game_length'] + f' ({POLISH_LABELS["ply"]})', fontsize=12)
    ax.set_title('Średnia długość partii wg algorytmu', fontsize=14, fontweight='bold')

    for bar, length in zip(bars, lengths):
        ax.text(length + 0.5, bar.get_y() + bar.get_height()/2,
                f'{length:.1f}', va='center', fontsize=10, fontweight='bold')

    plt.tight_layout()
    plt.savefig(output_dir / 'game_length.png', dpi=150)
    plt.savefig(output_dir / 'game_length.pdf')
    plt.close()


def plot_performance_radar(stats: dict[str, PlayerStats], output_dir: Path):
    """Radar chart comparing algorithm performance metrics."""
    algorithms = list(stats.keys())

    # Metrics: score%, win%, draw%, 1-loss%
    metrics = ['Wynik %', 'Wygrane %', 'Remisy %', '100 - Przegrane %']
    n_metrics = len(metrics)

    # Calculate values
    data = {}
    for algo in algorithms:
        s = stats[algo]
        total = s.games if s.games > 0 else 1
        data[algo] = [
            s.score_percent,
            s.win_percent,
            (s.draws / total) * 100,
            100 - (s.losses / total) * 100
        ]

    # Create radar chart
    angles = np.linspace(0, 2 * np.pi, n_metrics, endpoint=False).tolist()
    angles += angles[:1]

    fig, ax = plt.subplots(figsize=(10, 10), subplot_kw=dict(polar=True))

    for algo in algorithms:
        values = data[algo] + data[algo][:1]
        ax.plot(angles, values, 'o-', linewidth=2,
               label=get_algorithm_label(algo),
               color=get_algorithm_color(algo))
        ax.fill(angles, values, alpha=0.15, color=get_algorithm_color(algo))

    ax.set_xticks(angles[:-1])
    ax.set_xticklabels(metrics, fontsize=11)
    ax.set_ylim(0, 100)
    ax.set_title('Porównanie wielowymiarowe algorytmów\n(wyniki turniejowe)',
                 fontsize=14, fontweight='bold', y=1.08)
    ax.legend(loc='upper right', bbox_to_anchor=(1.3, 1.0))

    plt.tight_layout()
    plt.savefig(output_dir / 'performance_radar.png', dpi=150, bbox_inches='tight')
    plt.savefig(output_dir / 'performance_radar.pdf', bbox_inches='tight')
    plt.close()


def plot_head_to_head_detailed(games: list[GameResult], algorithms: list[str], output_dir: Path):
    """Detailed head-to-head results for each algorithm pair."""
    h2h = calculate_head_to_head(games, algorithms)

    # Create subplot grid
    n = len(algorithms)
    fig, axes = plt.subplots(n, 1, figsize=(12, 3*n))
    if n == 1:
        axes = [axes]

    for idx, algo in enumerate(algorithms):
        ax = axes[idx]
        opponents = [a for a in algorithms if a != algo]

        x = np.arange(len(opponents))
        width = 0.25

        wins = [h2h[algo][opp]['wins'] for opp in opponents]
        draws = [h2h[algo][opp]['draws'] for opp in opponents]
        losses = [h2h[algo][opp]['losses'] for opp in opponents]

        ax.bar(x - width, wins, width, label=POLISH_LABELS['wins'], color='#2ecc71')
        ax.bar(x, draws, width, label=POLISH_LABELS['draws'], color='#f39c12')
        ax.bar(x + width, losses, width, label=POLISH_LABELS['losses'], color='#e74c3c')

        ax.set_ylabel('Partie')
        ax.set_title(f'{get_algorithm_label(algo)} vs pozostałe algorytmy', fontsize=11, fontweight='bold')
        ax.set_xticks(x)
        ax.set_xticklabels([get_algorithm_label(o) for o in opponents], rotation=15, ha='right')

        if idx == 0:
            ax.legend(loc='upper right')

    plt.suptitle('Szczegółowe wyniki bezpośrednich pojedynków', fontsize=14, fontweight='bold', y=1.01)
    plt.tight_layout()
    plt.savefig(output_dir / 'head_to_head_detailed.png', dpi=150, bbox_inches='tight')
    plt.savefig(output_dir / 'head_to_head_detailed.pdf', bbox_inches='tight')
    plt.close()


# ============================================================================
# TEXT SUMMARY
# ============================================================================

def generate_summary_report(games: list[GameResult], algorithms: list[str], output_dir: Path):
    """Generate comprehensive text summary."""
    stats = calculate_overall_stats(games, algorithms)
    h2h = calculate_head_to_head(games, algorithms)

    lines = []
    lines.append("=" * 70)
    lines.append("ANALIZA PORÓWNAWCZA ALGORYTMÓW PRZESZUKIWANIA")
    lines.append("Silnik szachowy Aether - wyniki turnieju")
    lines.append("=" * 70)
    lines.append("")
    lines.append(f"Całkowita liczba partii: {len(games)}")
    lines.append(f"Testowane algorytmy: {', '.join([get_algorithm_label(a) for a in algorithms])}")
    lines.append("")

    # Overall ranking
    lines.append("-" * 70)
    lines.append("RANKING OGÓLNY (posortowany wg wyniku procentowego)")
    lines.append("-" * 70)

    sorted_algos = sorted(algorithms, key=lambda x: stats[x].score_percent, reverse=True)
    for i, algo in enumerate(sorted_algos, 1):
        s = stats[algo]
        lines.append(f"  {i}. {get_algorithm_label(algo):25} "
                    f"Wynik: {s.score:5.1f}/{s.games:3} ({s.score_percent:5.1f}%)  "
                    f"W/R/P: {s.wins:3}/{s.draws:3}/{s.losses:3}")

    # Head-to-head results
    lines.append("")
    lines.append("-" * 70)
    lines.append("WYNIKI BEZPOŚREDNICH POJEDYNKÓW")
    lines.append("-" * 70)

    for algo in sorted_algos:
        lines.append(f"\n  {get_algorithm_label(algo)}:")
        for opp in sorted_algos:
            if algo == opp:
                continue
            r = h2h[algo][opp]
            total = r['wins'] + r['draws'] + r['losses']
            if total > 0:
                score = r['wins'] + 0.5 * r['draws']
                pct = score / total * 100
                lines.append(f"    vs {get_algorithm_label(opp):25} "
                           f"W/R/P: {r['wins']}/{r['draws']}/{r['losses']}  "
                           f"Wynik: {score:.1f}/{total} ({pct:.1f}%)")

    # Conclusions
    lines.append("")
    lines.append("-" * 70)
    lines.append("WNIOSKI")
    lines.append("-" * 70)

    best_algo = sorted_algos[0]
    worst_algo = sorted_algos[-1]
    best_stats = stats[best_algo]
    worst_stats = stats[worst_algo]

    lines.append(f"  Najlepszy algorytm:  {get_algorithm_label(best_algo)} "
                f"({best_stats.score_percent:.1f}%)")
    lines.append(f"  Najsłabszy algorytm: {get_algorithm_label(worst_algo)} "
                f"({worst_stats.score_percent:.1f}%)")

    # Calculate dominance
    lines.append("")
    lines.append("  Dominacja (algorytm A dominuje B jeśli wynik > 50%):")
    for algo in sorted_algos:
        dominated = []
        for opp in algorithms:
            if algo == opp:
                continue
            r = h2h[algo][opp]
            total = r['wins'] + r['draws'] + r['losses']
            if total > 0:
                score_pct = (r['wins'] + 0.5 * r['draws']) / total * 100
                if score_pct > 50:
                    dominated.append(get_algorithm_label(opp))
        if dominated:
            lines.append(f"    {get_algorithm_label(algo)} dominuje: {', '.join(dominated)}")

    summary_text = "\n".join(lines)

    with open(output_dir / 'tournament_summary.txt', 'w', encoding='utf-8') as f:
        f.write(summary_text)

    print(summary_text)


# ============================================================================
# MAIN
# ============================================================================

def main():
    if len(sys.argv) < 2:
        print("Użycie: python visualize_elo_test.py <turniej.pgn> [katalog_wyjsciowy]")
        print("Przykład: python visualize_elo_test.py ../results/elo_test_20260127.pgn ../results/tournament_charts/")
        sys.exit(1)

    pgn_path = Path(sys.argv[1])
    output_dir = Path(sys.argv[2]) if len(sys.argv) > 2 else pgn_path.parent / 'tournament_charts'

    if not pgn_path.exists():
        print(f"Błąd: Nie znaleziono pliku: {pgn_path}")
        sys.exit(1)

    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"Parsowanie PGN: {pgn_path}")
    games = parse_pgn(pgn_path)
    print(f"Znaleziono {len(games)} partii")

    if not games:
        print("Nie znaleziono partii w pliku PGN")
        sys.exit(1)

    algorithms = get_all_players(games)
    print(f"Algorytmy: {', '.join([get_algorithm_label(a) for a in algorithms])}")

    # Calculate statistics
    print("\nObliczanie statystyk...")
    stats = calculate_overall_stats(games, algorithms)

    # Generate visualizations
    print("Generowanie wykresów...")

    plot_algorithm_comparison_bar(stats, output_dir)
    print("  - Porównanie algorytmów (słupkowy)")

    plot_score_percentage_horizontal(stats, output_dir)
    print("  - Wynik procentowy (horyzontalny)")

    plot_head_to_head_heatmap(games, algorithms, output_dir)
    print("  - Heatmapa head-to-head")

    plot_stacked_results(stats, output_dir)
    print("  - Wyniki skumulowane")

    plot_win_draw_loss_pie(stats, output_dir)
    print("  - Wykresy kołowe W/R/P")

    plot_tournament_crosstable(games, algorithms, output_dir)
    print("  - Tabela turniejowa")

    plot_algorithm_ranking(stats, output_dir)
    print("  - Ranking algorytmów")

    plot_game_length_comparison(stats, output_dir)
    print("  - Długość partii")

    plot_performance_radar(stats, output_dir)
    print("  - Wykres radarowy")

    plot_head_to_head_detailed(games, algorithms, output_dir)
    print("  - Szczegóły head-to-head")

    print("\nGenerowanie raportu...")
    generate_summary_report(games, algorithms, output_dir)

    print(f"\nWszystkie wykresy zapisano w: {output_dir}")
    print("Formaty: PNG (do podglądu), PDF (do pracy magisterskiej)")


if __name__ == "__main__":
    main()
