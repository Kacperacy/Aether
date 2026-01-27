#!/usr/bin/env python3
"""
Aether Chess Engine - Benchmark Results Visualization
For Master's Thesis: Comparative Analysis of Decision Tree Search Algorithms
"""

import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from pathlib import Path
import sys

# Konfiguracja stylu
plt.style.use('seaborn-v0_8-whitegrid')
sns.set_palette("husl")

def load_results(csv_path: str) -> pd.DataFrame:
    """Wczytaj wyniki z CSV"""
    df = pd.read_csv(csv_path)
    # Upewnij sie, ze kolumny sa poprawnie nazwane
    expected_cols = ['algorithm', 'position_id', 'phase', 'depth', 'nodes',
                     'time_ms', 'nps', 'ttfm_ms', 'best_move', 'score',
                     'branching_factor', 'stability']
    return df

def plot_nps_comparison(df: pd.DataFrame, output_dir: Path):
    """Wykres NPS (nodes per second) dla kazdego algorytmu"""
    fig, ax = plt.subplots(figsize=(12, 6))

    avg_nps = df.groupby('algorithm')['nps'].mean().sort_values(ascending=True)
    colors = sns.color_palette("husl", len(avg_nps))

    bars = ax.barh(avg_nps.index, avg_nps.values, color=colors)
    ax.set_xlabel('Average NPS (nodes/second)', fontsize=12)
    ax.set_title('Throughput Comparison: Nodes Per Second', fontsize=14, fontweight='bold')

    # Dodaj wartosci na slupkach
    for bar, val in zip(bars, avg_nps.values):
        ax.text(val + max(avg_nps) * 0.01, bar.get_y() + bar.get_height()/2,
                f'{val:,.0f}', va='center', fontsize=10)

    plt.tight_layout()
    plt.savefig(output_dir / 'nps_comparison.png', dpi=150)
    plt.close()

def plot_depth_comparison(df: pd.DataFrame, output_dir: Path):
    """Wykres glebokosci przeszukiwania"""
    fig, ax = plt.subplots(figsize=(12, 6))

    avg_depth = df.groupby('algorithm')['depth'].mean().sort_values(ascending=True)
    colors = sns.color_palette("husl", len(avg_depth))

    bars = ax.barh(avg_depth.index, avg_depth.values, color=colors)
    ax.set_xlabel('Average Search Depth', fontsize=12)
    ax.set_title('Search Depth Comparison', fontsize=14, fontweight='bold')

    for bar, val in zip(bars, avg_depth.values):
        ax.text(val + 0.1, bar.get_y() + bar.get_height()/2,
                f'{val:.1f}', va='center', fontsize=10)

    plt.tight_layout()
    plt.savefig(output_dir / 'depth_comparison.png', dpi=150)
    plt.close()

def plot_nodes_comparison(df: pd.DataFrame, output_dir: Path):
    """Wykres liczby wezlow"""
    fig, ax = plt.subplots(figsize=(12, 6))

    avg_nodes = df.groupby('algorithm')['nodes'].mean().sort_values(ascending=True)
    colors = sns.color_palette("husl", len(avg_nodes))

    bars = ax.barh(avg_nodes.index, avg_nodes.values, color=colors)
    ax.set_xlabel('Average Nodes Searched', fontsize=12)
    ax.set_title('Nodes Searched Comparison', fontsize=14, fontweight='bold')
    ax.set_xscale('log')

    plt.tight_layout()
    plt.savefig(output_dir / 'nodes_comparison.png', dpi=150)
    plt.close()

def plot_ttfm_comparison(df: pd.DataFrame, output_dir: Path):
    """Wykres TTFM (Time To First Move) - responsywnosc"""
    fig, ax = plt.subplots(figsize=(12, 6))

    avg_ttfm = df.groupby('algorithm')['ttfm_ms'].mean().sort_values(ascending=True)
    colors = sns.color_palette("husl", len(avg_ttfm))

    bars = ax.barh(avg_ttfm.index, avg_ttfm.values, color=colors)
    ax.set_xlabel('Average TTFM (ms)', fontsize=12)
    ax.set_title('Responsiveness: Time To First Move', fontsize=14, fontweight='bold')

    for bar, val in zip(bars, avg_ttfm.values):
        ax.text(val + max(avg_ttfm) * 0.01, bar.get_y() + bar.get_height()/2,
                f'{val:.1f}ms', va='center', fontsize=10)

    plt.tight_layout()
    plt.savefig(output_dir / 'ttfm_comparison.png', dpi=150)
    plt.close()

def plot_phase_comparison(df: pd.DataFrame, output_dir: Path):
    """Wykres porownawczy po fazach gry"""
    fig, axes = plt.subplots(1, 3, figsize=(18, 6))

    metrics = ['nps', 'depth', 'nodes']
    titles = ['NPS by Game Phase', 'Depth by Game Phase', 'Nodes by Game Phase']

    for ax, metric, title in zip(axes, metrics, titles):
        pivot = df.pivot_table(values=metric, index='algorithm', columns='phase', aggfunc='mean')
        pivot.plot(kind='bar', ax=ax, width=0.8)
        ax.set_title(title, fontsize=12, fontweight='bold')
        ax.set_xlabel('')
        ax.set_ylabel(metric.upper())
        ax.legend(title='Phase')
        ax.tick_params(axis='x', rotation=45)

        if metric == 'nodes':
            ax.set_yscale('log')

    plt.tight_layout()
    plt.savefig(output_dir / 'phase_comparison.png', dpi=150)
    plt.close()

def plot_boxplots(df: pd.DataFrame, output_dir: Path):
    """Box plots dla rozkladu metryk"""
    fig, axes = plt.subplots(2, 2, figsize=(14, 12))

    metrics = [('nps', 'NPS Distribution'), ('depth', 'Depth Distribution'),
               ('nodes', 'Nodes Distribution'), ('ttfm_ms', 'TTFM Distribution')]

    for ax, (metric, title) in zip(axes.flatten(), metrics):
        sns.boxplot(data=df, x='algorithm', y=metric, ax=ax)
        ax.set_title(title, fontsize=12, fontweight='bold')
        ax.tick_params(axis='x', rotation=45)

        if metric == 'nodes':
            ax.set_yscale('log')

    plt.tight_layout()
    plt.savefig(output_dir / 'distributions.png', dpi=150)
    plt.close()

def generate_summary(df: pd.DataFrame, output_dir: Path):
    """Generuj podsumowanie tekstowe"""
    summary = []
    summary.append("=" * 60)
    summary.append("BENCHMARK RESULTS SUMMARY")
    summary.append("=" * 60)
    summary.append("")

    summary.append(f"Total positions tested: {len(df) // df['algorithm'].nunique()}")
    summary.append(f"Algorithms tested: {df['algorithm'].nunique()}")
    summary.append(f"Phases covered: {', '.join(df['phase'].unique())}")
    summary.append("")

    summary.append("-" * 60)
    summary.append("AVERAGE METRICS BY ALGORITHM")
    summary.append("-" * 60)

    for algo in df['algorithm'].unique():
        algo_df = df[df['algorithm'] == algo]
        summary.append(f"\n{algo}:")
        summary.append(f"  NPS:    {algo_df['nps'].mean():>12,.0f}")
        summary.append(f"  Depth:  {algo_df['depth'].mean():>12.1f}")
        summary.append(f"  Nodes:  {algo_df['nodes'].mean():>12,.0f}")
        summary.append(f"  TTFM:   {algo_df['ttfm_ms'].mean():>12.1f} ms")

    summary.append("")
    summary.append("-" * 60)
    summary.append("METRICS BY GAME PHASE")
    summary.append("-" * 60)

    for phase in df['phase'].unique():
        phase_df = df[df['phase'] == phase]
        summary.append(f"\n{phase.upper()}:")
        for algo in df['algorithm'].unique():
            algo_phase = phase_df[phase_df['algorithm'] == algo]
            if len(algo_phase) > 0:
                summary.append(f"  {algo}: NPS={algo_phase['nps'].mean():,.0f}, "
                             f"Depth={algo_phase['depth'].mean():.1f}")

    summary_text = "\n".join(summary)

    with open(output_dir / 'summary.txt', 'w') as f:
        f.write(summary_text)

    print(summary_text)

def main():
    if len(sys.argv) < 2:
        print("Usage: python visualize_results.py <results.csv> [output_dir]")
        print("Example: python visualize_results.py results/comparison_20260126.csv charts/")
        sys.exit(1)

    csv_path = Path(sys.argv[1])
    output_dir = Path(sys.argv[2]) if len(sys.argv) > 2 else csv_path.parent / 'charts'

    if not csv_path.exists():
        print(f"Error: File not found: {csv_path}")
        sys.exit(1)

    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"Loading results from: {csv_path}")
    df = load_results(csv_path)
    print(f"Loaded {len(df)} records")
    print(f"Algorithms: {df['algorithm'].unique().tolist()}")
    print(f"Phases: {df['phase'].unique().tolist()}")
    print()

    print("Generating visualizations...")
    plot_nps_comparison(df, output_dir)
    print("  - NPS comparison saved")

    plot_depth_comparison(df, output_dir)
    print("  - Depth comparison saved")

    plot_nodes_comparison(df, output_dir)
    print("  - Nodes comparison saved")

    plot_ttfm_comparison(df, output_dir)
    print("  - TTFM comparison saved")

    plot_phase_comparison(df, output_dir)
    print("  - Phase comparison saved")

    plot_boxplots(df, output_dir)
    print("  - Distribution plots saved")

    print()
    generate_summary(df, output_dir)

    print()
    print(f"All charts saved to: {output_dir}")

if __name__ == "__main__":
    main()
