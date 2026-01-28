#!/usr/bin/env python3
"""
Aether Chess Engine - Benchmark Results Visualization
For Master's Thesis: Comparative Analysis of Decision Tree Search Algorithms

Generates charts for comparing:
- Alpha-Beta, NegaScout, MTD(f), MCTS algorithms
- Performance metrics: NPS, depth, nodes, TTFM
- Analysis across game phases and time controls
"""

import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import numpy as np
from pathlib import Path
import sys
import glob
import io

# Fix Windows console encoding for Polish characters
if sys.platform == 'win32':
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8', errors='replace')

# Configuration for thesis-quality plots
plt.style.use('seaborn-v0_8-whitegrid')
plt.rcParams['font.size'] = 11
plt.rcParams['axes.titlesize'] = 13
plt.rcParams['axes.labelsize'] = 12
plt.rcParams['figure.dpi'] = 150

# Polish labels for thesis
POLISH_LABELS = {
    'nps': 'Węzły na sekundę (NPS)',
    'depth': 'Głębokość przeszukiwania',
    'nodes': 'Liczba przeszukanych węzłów',
    'ttfm_ms': 'Czas do pierwszego ruchu (ms)',
    'time_ms': 'Czas obliczeń (ms)',
    'branching_factor': 'Współczynnik rozgałęzienia',
    'score': 'Ocena pozycji',
    'algorithm': 'Algorytm',
    'phase': 'Faza gry',
    'opening': 'Otwarcie',
    'middlegame': 'Środek gry',
    'endgame': 'Końcówka',
    'time_limit': 'Limit czasu',
}

# Algorithm display names (Polish)
ALGORITHM_NAMES = {
    'Pure Alpha-Beta': 'Alpha-Beta (czysty)',
    'Full Alpha-Beta': 'Alpha-Beta (pełny)',
    'NegaScout': 'NegaScout',
    'MTD(f)': 'MTD(f)',
    'MCTS': 'MCTS',
    'Classic MCTS': 'MCTS (klasyczny)',
}

# Colors for algorithms - consistent across all charts
ALGORITHM_COLORS = {
    'Pure Alpha-Beta': '#27ae60',
    'Full Alpha-Beta': '#2ecc71',
    'NegaScout': '#3498db',
    'MTD(f)': '#9b59b6',
    'MCTS': '#e74c3c',
    'Classic MCTS': '#c0392b',
}

# Phase colors
PHASE_COLORS = {
    'opening': '#3498db',
    'middlegame': '#f39c12',
    'endgame': '#e74c3c',
}


def get_algorithm_color(algo):
    """Get color for algorithm, with fallback."""
    return ALGORITHM_COLORS.get(algo, '#7f8c8d')


def get_algorithm_label(algo):
    """Get Polish label for algorithm."""
    return ALGORITHM_NAMES.get(algo, algo)


def get_phase_label(phase):
    """Get Polish label for game phase."""
    return POLISH_LABELS.get(phase, phase.capitalize())


def load_all_results(results_dir: Path) -> pd.DataFrame:
    """Load results from all time limit folders."""
    all_data = []

    for time_folder in ['500ms', '2000ms', '5000ms']:
        folder_path = results_dir / time_folder
        if not folder_path.exists():
            continue

        time_limit = int(time_folder.replace('ms', ''))

        # Find comparison CSV file
        csv_files = list(folder_path.glob('comparison_*.csv'))
        if csv_files:
            df = pd.read_csv(csv_files[0])
            df['time_limit'] = time_limit
            all_data.append(df)

    if not all_data:
        raise ValueError(f"No CSV files found in {results_dir}")

    combined = pd.concat(all_data, ignore_index=True)
    return combined


def load_single_result(csv_path: Path) -> pd.DataFrame:
    """Load a single CSV result file."""
    return pd.read_csv(csv_path)


# =============================================================================
# BASIC COMPARISON CHARTS
# =============================================================================

def plot_nps_comparison(df: pd.DataFrame, output_dir: Path):
    """Horizontal bar chart: NPS comparison."""
    fig, ax = plt.subplots(figsize=(12, 6))

    avg_nps = df.groupby('algorithm')['nps'].mean().sort_values(ascending=True)
    labels = [get_algorithm_label(a) for a in avg_nps.index]
    colors = [get_algorithm_color(a) for a in avg_nps.index]

    bars = ax.barh(labels, avg_nps.values, color=colors)
    ax.set_xlabel(POLISH_LABELS['nps'], fontsize=12)
    ax.set_title('Porównanie przepustowości algorytmów (NPS)', fontsize=14, fontweight='bold')

    # Add value labels
    for bar, val in zip(bars, avg_nps.values):
        ax.text(val + max(avg_nps) * 0.01, bar.get_y() + bar.get_height()/2,
                f'{val:,.0f}', va='center', fontsize=10, fontweight='bold')

    ax.set_xlim(0, max(avg_nps) * 1.15)
    plt.tight_layout()
    plt.savefig(output_dir / 'nps_comparison.png', dpi=150)
    plt.savefig(output_dir / 'nps_comparison.pdf')
    plt.close()


def plot_depth_comparison(df: pd.DataFrame, output_dir: Path):
    """Horizontal bar chart: Search depth comparison."""
    fig, ax = plt.subplots(figsize=(12, 6))

    avg_depth = df.groupby('algorithm')['depth'].mean().sort_values(ascending=True)
    labels = [get_algorithm_label(a) for a in avg_depth.index]
    colors = [get_algorithm_color(a) for a in avg_depth.index]

    bars = ax.barh(labels, avg_depth.values, color=colors)
    ax.set_xlabel(POLISH_LABELS['depth'], fontsize=12)
    ax.set_title('Porównanie głębokości przeszukiwania', fontsize=14, fontweight='bold')

    for bar, val in zip(bars, avg_depth.values):
        ax.text(val + 0.2, bar.get_y() + bar.get_height()/2,
                f'{val:.1f}', va='center', fontsize=10, fontweight='bold')

    ax.set_xlim(0, max(avg_depth) * 1.15)
    plt.tight_layout()
    plt.savefig(output_dir / 'depth_comparison.png', dpi=150)
    plt.savefig(output_dir / 'depth_comparison.pdf')
    plt.close()


def plot_ttfm_comparison(df: pd.DataFrame, output_dir: Path):
    """Horizontal bar chart: Time to First Move (responsiveness)."""
    fig, ax = plt.subplots(figsize=(12, 6))

    avg_ttfm = df.groupby('algorithm')['ttfm_ms'].mean().sort_values(ascending=False)
    labels = [get_algorithm_label(a) for a in avg_ttfm.index]
    colors = [get_algorithm_color(a) for a in avg_ttfm.index]

    bars = ax.barh(labels, avg_ttfm.values, color=colors)
    ax.set_xlabel(POLISH_LABELS['ttfm_ms'], fontsize=12)
    ax.set_title('Responsywność algorytmów (czas do pierwszego ruchu)', fontsize=14, fontweight='bold')

    for bar, val in zip(bars, avg_ttfm.values):
        ax.text(val + max(avg_ttfm) * 0.01, bar.get_y() + bar.get_height()/2,
                f'{val:.1f} ms', va='center', fontsize=10, fontweight='bold')

    ax.set_xlim(0, max(avg_ttfm) * 1.15)
    plt.tight_layout()
    plt.savefig(output_dir / 'ttfm_comparison.png', dpi=150)
    plt.savefig(output_dir / 'ttfm_comparison.pdf')
    plt.close()


def plot_nodes_comparison(df: pd.DataFrame, output_dir: Path):
    """Horizontal bar chart: Nodes searched (log scale)."""
    fig, ax = plt.subplots(figsize=(12, 6))

    avg_nodes = df.groupby('algorithm')['nodes'].mean().sort_values(ascending=True)
    labels = [get_algorithm_label(a) for a in avg_nodes.index]
    colors = [get_algorithm_color(a) for a in avg_nodes.index]

    bars = ax.barh(labels, avg_nodes.values, color=colors)
    ax.set_xlabel(POLISH_LABELS['nodes'] + ' (skala logarytmiczna)', fontsize=12)
    ax.set_title('Liczba przeszukanych węzłów', fontsize=14, fontweight='bold')
    ax.set_xscale('log')

    plt.tight_layout()
    plt.savefig(output_dir / 'nodes_comparison.png', dpi=150)
    plt.savefig(output_dir / 'nodes_comparison.pdf')
    plt.close()


# =============================================================================
# PHASE COMPARISON CHARTS
# =============================================================================

def plot_phase_comparison_grouped(df: pd.DataFrame, output_dir: Path):
    """Grouped bar chart: Metrics by game phase."""
    phases = df['phase'].unique()
    if len(phases) <= 1:
        return  # Skip if only one phase

    algorithms = df['algorithm'].unique()

    fig, axes = plt.subplots(1, 3, figsize=(18, 6))

    metrics = [('nps', 'NPS wg fazy gry'),
               ('depth', 'Głębokość wg fazy gry'),
               ('ttfm_ms', 'TTFM wg fazy gry')]

    for ax, (metric, title) in zip(axes, metrics):
        x = np.arange(len(algorithms))
        width = 0.25

        for i, phase in enumerate(sorted(phases)):
            phase_data = df[df['phase'] == phase]
            values = [phase_data[phase_data['algorithm'] == algo][metric].mean()
                     for algo in algorithms]
            offset = (i - len(phases)/2 + 0.5) * width
            bars = ax.bar(x + offset, values, width,
                         label=get_phase_label(phase),
                         color=PHASE_COLORS.get(phase, f'C{i}'))

        ax.set_ylabel(POLISH_LABELS.get(metric, metric))
        ax.set_title(title, fontsize=12, fontweight='bold')
        ax.set_xticks(x)
        ax.set_xticklabels([get_algorithm_label(a) for a in algorithms], rotation=20, ha='right')
        ax.legend(title='Faza gry')

        if metric == 'nodes':
            ax.set_yscale('log')

    plt.tight_layout()
    plt.savefig(output_dir / 'phase_comparison.png', dpi=150)
    plt.savefig(output_dir / 'phase_comparison.pdf')
    plt.close()


def plot_phase_heatmap(df: pd.DataFrame, output_dir: Path, metric='nps'):
    """Heatmap: Algorithm performance by phase."""
    phases = df['phase'].unique()
    if len(phases) <= 1:
        return

    pivot = df.pivot_table(values=metric, index='algorithm', columns='phase', aggfunc='mean')
    pivot = pivot.reindex(columns=sorted(pivot.columns))

    fig, ax = plt.subplots(figsize=(10, 7))

    # Normalize for better visualization
    data = pivot.values

    im = ax.imshow(data, cmap='YlGnBu', aspect='auto')

    # Labels
    ax.set_xticks(np.arange(len(pivot.columns)))
    ax.set_yticks(np.arange(len(pivot.index)))
    ax.set_xticklabels([get_phase_label(p) for p in pivot.columns])
    ax.set_yticklabels([get_algorithm_label(a) for a in pivot.index])

    # Add text annotations
    for i in range(len(pivot.index)):
        for j in range(len(pivot.columns)):
            val = data[i, j]
            text = f'{val:,.0f}' if val > 1000 else f'{val:.1f}'
            color = 'white' if val > data.max() * 0.6 else 'black'
            ax.text(j, i, text, ha='center', va='center', color=color, fontsize=10)

    ax.set_title(f'{POLISH_LABELS.get(metric, metric)} w zależności od fazy gry',
                 fontsize=14, fontweight='bold')

    cbar = plt.colorbar(im, ax=ax)
    cbar.set_label(POLISH_LABELS.get(metric, metric))

    plt.tight_layout()
    plt.savefig(output_dir / f'phase_heatmap_{metric}.png', dpi=150)
    plt.savefig(output_dir / f'phase_heatmap_{metric}.pdf')
    plt.close()


# =============================================================================
# TIME LIMIT COMPARISON CHARTS
# =============================================================================

def plot_time_limit_comparison(df: pd.DataFrame, output_dir: Path):
    """Compare algorithm performance across different time limits."""
    if 'time_limit' not in df.columns:
        return

    time_limits = sorted(df['time_limit'].unique())
    if len(time_limits) <= 1:
        return

    algorithms = df['algorithm'].unique()

    fig, axes = plt.subplots(2, 2, figsize=(14, 12))
    metrics = [('nps', 'NPS'), ('depth', 'Głębokość'),
               ('nodes', 'Węzły'), ('ttfm_ms', 'TTFM')]

    for ax, (metric, title) in zip(axes.flatten(), metrics):
        for algo in algorithms:
            algo_data = df[df['algorithm'] == algo]
            means = [algo_data[algo_data['time_limit'] == t][metric].mean()
                    for t in time_limits]
            ax.plot(time_limits, means, 'o-',
                   label=get_algorithm_label(algo),
                   color=get_algorithm_color(algo),
                   linewidth=2, markersize=8)

        ax.set_xlabel('Limit czasu (ms)', fontsize=11)
        ax.set_ylabel(POLISH_LABELS.get(metric, metric), fontsize=11)
        ax.set_title(f'{title} w zależności od limitu czasu', fontsize=12, fontweight='bold')
        ax.legend(loc='best', fontsize=9)
        ax.set_xticks(time_limits)
        ax.set_xticklabels([f'{t}ms' for t in time_limits])

        if metric == 'nodes':
            ax.set_yscale('log')

    plt.tight_layout()
    plt.savefig(output_dir / 'time_limit_comparison.png', dpi=150)
    plt.savefig(output_dir / 'time_limit_comparison.pdf')
    plt.close()


def plot_depth_vs_time(df: pd.DataFrame, output_dir: Path):
    """Line chart: Depth achieved vs time limit."""
    if 'time_limit' not in df.columns:
        return

    time_limits = sorted(df['time_limit'].unique())
    if len(time_limits) <= 1:
        return

    fig, ax = plt.subplots(figsize=(12, 7))

    algorithms = df['algorithm'].unique()

    for algo in algorithms:
        algo_data = df[df['algorithm'] == algo]
        depths = [algo_data[algo_data['time_limit'] == t]['depth'].mean()
                 for t in time_limits]
        ax.plot(time_limits, depths, 'o-',
               label=get_algorithm_label(algo),
               color=get_algorithm_color(algo),
               linewidth=2.5, markersize=10)

    ax.set_xlabel('Limit czasu (ms)', fontsize=12)
    ax.set_ylabel(POLISH_LABELS['depth'], fontsize=12)
    ax.set_title('Głębokość przeszukiwania w zależności od limitu czasu',
                 fontsize=14, fontweight='bold')
    ax.legend(loc='best', fontsize=10)
    ax.set_xticks(time_limits)
    ax.set_xticklabels([f'{t}ms' for t in time_limits])
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    plt.savefig(output_dir / 'depth_vs_time.png', dpi=150)
    plt.savefig(output_dir / 'depth_vs_time.pdf')
    plt.close()


# =============================================================================
# DISTRIBUTION CHARTS
# =============================================================================

def plot_boxplots(df: pd.DataFrame, output_dir: Path):
    """Box plots for metric distributions."""
    fig, axes = plt.subplots(2, 2, figsize=(14, 12))

    metrics = [('nps', 'Rozkład NPS'),
               ('depth', 'Rozkład głębokości'),
               ('nodes', 'Rozkład liczby węzłów'),
               ('ttfm_ms', 'Rozkład TTFM')]

    # Sort algorithms by median NPS for consistent ordering
    algo_order = df.groupby('algorithm')['nps'].median().sort_values(ascending=False).index.tolist()

    for ax, (metric, title) in zip(axes.flatten(), metrics):
        data_to_plot = [df[df['algorithm'] == algo][metric].values for algo in algo_order]
        labels = [get_algorithm_label(a) for a in algo_order]
        colors = [get_algorithm_color(a) for a in algo_order]

        bp = ax.boxplot(data_to_plot, tick_labels=labels, patch_artist=True)

        for patch, color in zip(bp['boxes'], colors):
            patch.set_facecolor(color)
            patch.set_alpha(0.7)

        ax.set_title(title, fontsize=12, fontweight='bold')
        ax.set_ylabel(POLISH_LABELS.get(metric, metric))
        ax.tick_params(axis='x', rotation=20)

        if metric == 'nodes':
            ax.set_yscale('log')

    plt.tight_layout()
    plt.savefig(output_dir / 'distributions.png', dpi=150)
    plt.savefig(output_dir / 'distributions.pdf')
    plt.close()


def plot_violin_plots(df: pd.DataFrame, output_dir: Path):
    """Violin plots for detailed distributions."""
    import warnings
    warnings.filterwarnings('ignore')

    fig, axes = plt.subplots(2, 2, figsize=(14, 12))

    metrics = [('nps', 'Rozkład NPS'),
               ('depth', 'Rozkład głębokości'),
               ('ttfm_ms', 'Rozkład TTFM'),
               ('branching_factor', 'Rozkład współczynnika rozgałęzienia')]

    algorithms = df['algorithm'].unique()

    for ax, (metric, title) in zip(axes.flatten(), metrics):
        data = [df[df['algorithm'] == algo][metric].dropna().values for algo in algorithms]
        labels = [get_algorithm_label(a) for a in algorithms]

        parts = ax.violinplot(data, showmeans=True, showmedians=True)

        for i, pc in enumerate(parts['bodies']):
            pc.set_facecolor(get_algorithm_color(algorithms[i]))
            pc.set_alpha(0.7)

        ax.set_xticks(range(1, len(algorithms) + 1))
        ax.set_xticklabels(labels, rotation=20, ha='right')
        ax.set_title(title, fontsize=12, fontweight='bold')
        ax.set_ylabel(POLISH_LABELS.get(metric, metric))

    plt.tight_layout()
    plt.savefig(output_dir / 'violin_plots.png', dpi=150)
    plt.savefig(output_dir / 'violin_plots.pdf')
    plt.close()


# =============================================================================
# COMPREHENSIVE SUMMARY CHART
# =============================================================================

def plot_radar_chart(df: pd.DataFrame, output_dir: Path):
    """Radar chart comparing algorithms across multiple metrics."""
    algorithms = df['algorithm'].unique()

    # Metrics to compare (normalized)
    metrics = ['nps', 'depth', 'nodes', 'ttfm_ms', 'branching_factor']
    metric_labels = ['NPS', 'Głębokość', 'Węzły', 'TTFM', 'Rozgałęzienie']

    # Calculate averages
    data = {}
    for algo in algorithms:
        algo_df = df[df['algorithm'] == algo]
        data[algo] = [algo_df[m].mean() for m in metrics]

    # Normalize (0-1 scale, higher is better for NPS/depth, lower is better for TTFM)
    normalized = {}
    for algo in algorithms:
        normalized[algo] = []
        for i, metric in enumerate(metrics):
            all_vals = [data[a][i] for a in algorithms]
            min_val, max_val = min(all_vals), max(all_vals)
            if max_val == min_val:
                norm = 0.5
            else:
                norm = (data[algo][i] - min_val) / (max_val - min_val)

            # Invert for metrics where lower is better
            if metric in ['ttfm_ms']:
                norm = 1 - norm

            normalized[algo].append(norm)

    # Create radar chart
    angles = np.linspace(0, 2 * np.pi, len(metrics), endpoint=False).tolist()
    angles += angles[:1]  # Complete the circle

    fig, ax = plt.subplots(figsize=(10, 10), subplot_kw=dict(polar=True))

    for algo in algorithms:
        values = normalized[algo] + normalized[algo][:1]
        ax.plot(angles, values, 'o-', linewidth=2,
               label=get_algorithm_label(algo),
               color=get_algorithm_color(algo))
        ax.fill(angles, values, alpha=0.15, color=get_algorithm_color(algo))

    ax.set_xticks(angles[:-1])
    ax.set_xticklabels(metric_labels, fontsize=11)
    ax.set_ylim(0, 1)
    ax.set_title('Porównanie wielowymiarowe algorytmów\n(wartości znormalizowane, wyżej = lepiej)',
                 fontsize=14, fontweight='bold', y=1.08)
    ax.legend(loc='upper right', bbox_to_anchor=(1.3, 1.0))

    plt.tight_layout()
    plt.savefig(output_dir / 'radar_comparison.png', dpi=150, bbox_inches='tight')
    plt.savefig(output_dir / 'radar_comparison.pdf', bbox_inches='tight')
    plt.close()


def plot_summary_ranking(df: pd.DataFrame, output_dir: Path):
    """Summary ranking chart with multiple metrics."""
    fig, axes = plt.subplots(1, 3, figsize=(16, 6))

    # 1. NPS ranking
    ax = axes[0]
    avg_nps = df.groupby('algorithm')['nps'].mean().sort_values(ascending=True)
    labels = [get_algorithm_label(a) for a in avg_nps.index]
    colors = [get_algorithm_color(a) for a in avg_nps.index]
    bars = ax.barh(labels, avg_nps.values, color=colors)
    ax.set_xlabel('Średnie NPS')
    ax.set_title('Ranking: Przepustowość', fontsize=12, fontweight='bold')
    for bar, val in zip(bars, avg_nps.values):
        ax.text(val * 1.01, bar.get_y() + bar.get_height()/2,
                f'{val:,.0f}', va='center', fontsize=9)

    # 2. Depth ranking
    ax = axes[1]
    avg_depth = df.groupby('algorithm')['depth'].mean().sort_values(ascending=True)
    labels = [get_algorithm_label(a) for a in avg_depth.index]
    colors = [get_algorithm_color(a) for a in avg_depth.index]
    bars = ax.barh(labels, avg_depth.values, color=colors)
    ax.set_xlabel('Średnia głębokość')
    ax.set_title('Ranking: Głębokość', fontsize=12, fontweight='bold')
    for bar, val in zip(bars, avg_depth.values):
        ax.text(val + 0.1, bar.get_y() + bar.get_height()/2,
                f'{val:.1f}', va='center', fontsize=9)

    # 3. TTFM ranking (inverted - lower is better)
    ax = axes[2]
    avg_ttfm = df.groupby('algorithm')['ttfm_ms'].mean().sort_values(ascending=False)
    labels = [get_algorithm_label(a) for a in avg_ttfm.index]
    colors = [get_algorithm_color(a) for a in avg_ttfm.index]
    bars = ax.barh(labels, avg_ttfm.values, color=colors)
    ax.set_xlabel('Średni TTFM (ms)')
    ax.set_title('Ranking: Responsywność\n(niżej = lepiej)', fontsize=12, fontweight='bold')
    for bar, val in zip(bars, avg_ttfm.values):
        ax.text(val * 1.01, bar.get_y() + bar.get_height()/2,
                f'{val:.1f}', va='center', fontsize=9)

    plt.suptitle('Podsumowanie rankingowe algorytmów przeszukiwania',
                 fontsize=14, fontweight='bold', y=1.02)
    plt.tight_layout()
    plt.savefig(output_dir / 'summary_ranking.png', dpi=150, bbox_inches='tight')
    plt.savefig(output_dir / 'summary_ranking.pdf', bbox_inches='tight')
    plt.close()


# =============================================================================
# TEXT SUMMARY
# =============================================================================

def generate_summary(df: pd.DataFrame, output_dir: Path):
    """Generate comprehensive text summary."""
    lines = []
    lines.append("=" * 70)
    lines.append("ANALIZA PORÓWNAWCZA ALGORYTMÓW PRZESZUKIWANIA")
    lines.append("Silnik szachowy Aether - wyniki benchmarków")
    lines.append("=" * 70)
    lines.append("")

    # Basic stats
    algorithms = df['algorithm'].unique()
    phases = df['phase'].unique()
    time_limits = df['time_limit'].unique() if 'time_limit' in df.columns else []

    lines.append(f"Liczba pozycji testowych: {df['position_id'].nunique()}")
    lines.append(f"Testowane algorytmy: {len(algorithms)}")
    lines.append(f"Fazy gry: {', '.join(phases)}")
    if len(time_limits) > 0:
        lines.append(f"Limity czasu: {', '.join([f'{t}ms' for t in sorted(time_limits)])}")
    lines.append("")

    # Algorithm ranking
    lines.append("-" * 70)
    lines.append("RANKING ALGORYTMÓW (wg średniego NPS)")
    lines.append("-" * 70)

    avg_nps = df.groupby('algorithm')['nps'].mean().sort_values(ascending=False)
    for i, (algo, nps) in enumerate(avg_nps.items(), 1):
        algo_df = df[df['algorithm'] == algo]
        depth = algo_df['depth'].mean()
        nodes = algo_df['nodes'].mean()
        ttfm = algo_df['ttfm_ms'].mean()
        lines.append(f"  {i}. {get_algorithm_label(algo):25}")
        lines.append(f"     NPS: {nps:>12,.0f}  |  Głębokość: {depth:>6.1f}  |  "
                    f"Węzły: {nodes:>12,.0f}  |  TTFM: {ttfm:>8.2f}ms")

    # Phase analysis
    if len(phases) > 1:
        lines.append("")
        lines.append("-" * 70)
        lines.append("ANALIZA WG FAZ GRY")
        lines.append("-" * 70)

        for phase in sorted(phases):
            phase_df = df[df['phase'] == phase]
            lines.append(f"\n  {get_phase_label(phase).upper()}:")

            phase_nps = phase_df.groupby('algorithm')['nps'].mean().sort_values(ascending=False)
            for algo, nps in phase_nps.items():
                algo_phase = phase_df[phase_df['algorithm'] == algo]
                depth = algo_phase['depth'].mean()
                lines.append(f"    {get_algorithm_label(algo):25} "
                           f"NPS: {nps:>10,.0f}  Głębokość: {depth:>5.1f}")

    # Time limit analysis
    if len(time_limits) > 1:
        lines.append("")
        lines.append("-" * 70)
        lines.append("ANALIZA WG LIMITU CZASU")
        lines.append("-" * 70)

        for tl in sorted(time_limits):
            tl_df = df[df['time_limit'] == tl]
            lines.append(f"\n  LIMIT: {tl}ms")

            tl_depth = tl_df.groupby('algorithm')['depth'].mean().sort_values(ascending=False)
            for algo, depth in tl_depth.items():
                algo_tl = tl_df[tl_df['algorithm'] == algo]
                nps = algo_tl['nps'].mean()
                lines.append(f"    {get_algorithm_label(algo):25} "
                           f"Głębokość: {depth:>5.1f}  NPS: {nps:>10,.0f}")

    # Conclusions
    lines.append("")
    lines.append("-" * 70)
    lines.append("WNIOSKI")
    lines.append("-" * 70)

    best_nps = avg_nps.idxmax()
    best_depth = df.groupby('algorithm')['depth'].mean().idxmax()
    best_ttfm = df.groupby('algorithm')['ttfm_ms'].mean().idxmin()

    lines.append(f"  Najwyższa przepustowość (NPS): {get_algorithm_label(best_nps)}")
    lines.append(f"  Najgłębsze przeszukiwanie:    {get_algorithm_label(best_depth)}")
    lines.append(f"  Najlepsza responsywność:      {get_algorithm_label(best_ttfm)}")

    summary_text = "\n".join(lines)

    with open(output_dir / 'benchmark_summary.txt', 'w', encoding='utf-8') as f:
        f.write(summary_text)

    print(summary_text)


# =============================================================================
# MAIN
# =============================================================================

def main():
    if len(sys.argv) < 2:
        print("Użycie:")
        print("  python visualize_results.py <results_dir>")
        print("  python visualize_results.py <results.csv> [output_dir]")
        print("")
        print("Przykłady:")
        print("  python visualize_results.py ../results/")
        print("  python visualize_results.py ../results/500ms/comparison_20260127.csv charts/")
        sys.exit(1)

    input_path = Path(sys.argv[1])

    # Determine if input is a directory or a file
    if input_path.is_dir():
        # Load all results from time limit folders
        print(f"Wczytywanie wyników z katalogu: {input_path}")
        df = load_all_results(input_path)
        output_dir = input_path / 'charts'
    else:
        # Load single CSV file
        print(f"Wczytywanie wyników z pliku: {input_path}")
        df = load_single_result(input_path)
        output_dir = Path(sys.argv[2]) if len(sys.argv) > 2 else input_path.parent / 'charts'

    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"Wczytano {len(df)} rekordów")
    print(f"Algorytmy: {df['algorithm'].unique().tolist()}")
    print(f"Fazy: {df['phase'].unique().tolist()}")
    if 'time_limit' in df.columns:
        print(f"Limity czasu: {sorted(df['time_limit'].unique().tolist())}")
    print()

    print("Generowanie wykresów...")

    # Basic comparisons
    plot_nps_comparison(df, output_dir)
    print("  - Porównanie NPS")

    plot_depth_comparison(df, output_dir)
    print("  - Porównanie głębokości")

    plot_ttfm_comparison(df, output_dir)
    print("  - Porównanie TTFM")

    plot_nodes_comparison(df, output_dir)
    print("  - Porównanie węzłów")

    # Phase analysis
    plot_phase_comparison_grouped(df, output_dir)
    print("  - Porównanie wg faz gry")

    plot_phase_heatmap(df, output_dir, 'nps')
    print("  - Heatmapa NPS wg faz")

    plot_phase_heatmap(df, output_dir, 'depth')
    print("  - Heatmapa głębokości wg faz")

    # Time limit analysis
    plot_time_limit_comparison(df, output_dir)
    print("  - Porównanie wg limitów czasu")

    plot_depth_vs_time(df, output_dir)
    print("  - Głębokość vs czas")

    # Distributions
    plot_boxplots(df, output_dir)
    print("  - Wykresy pudełkowe")

    plot_violin_plots(df, output_dir)
    print("  - Wykresy skrzypcowe")

    # Summary
    plot_radar_chart(df, output_dir)
    print("  - Wykres radarowy")

    plot_summary_ranking(df, output_dir)
    print("  - Ranking podsumowujący")

    print()
    generate_summary(df, output_dir)

    print()
    print(f"Wszystkie wykresy zapisano w: {output_dir}")
    print("Formaty: PNG (podgląd), PDF (do pracy magisterskiej)")


if __name__ == "__main__":
    main()
