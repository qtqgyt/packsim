import subprocess
import re
import os
import sys

try:
    import matplotlib.pyplot as plt
    HAS_MATPLOTLIB = True
except ImportError:
    HAS_MATPLOTLIB = False

def run_benchmark(packs, threads):
    cmd = [
        "./cli/target/release/packopener-cli",
        "benchmark",
        str(packs),
        "--backend", "cpu",
        "--threads", str(threads)
    ]
    env = os.environ.copy()
    cwd = os.getcwd()
    env["LD_LIBRARY_PATH"] = cwd + ":" + env.get("LD_LIBRARY_PATH", "")

    try:
        # Some benchmarks might output to stdout, some to stderr, so capture both
        result = subprocess.run(cmd, env=env, capture_output=True, text=True)
        output = result.stderr + result.stdout

        # Parse output like: "10.00K packs, CPU backend (4 threads), 0.00s = 11.58M packs/s"
        match = re.search(r"([\d\.]+)s = ([\d\.]+)([KMGT]?) packs/s", output)
        if match:
            throughput = float(match.group(2))
            unit = match.group(3)
            multiplier = {"": 1, "K": 1e3, "M": 1e6, "G": 1e9, "T": 1e12}.get(unit, 1)
            return throughput * multiplier
        else:
            print(f"Could not parse output for {packs} packs, {threads} threads:\n{output}")
            return 0.0
    except Exception as e:
        print(f"Benchmark failed for {packs} packs, {threads} threads:\n{e}")
        return 0.0

def generate_svg(data, thread_counts, filename):
    # Determine max values for scaling
    max_speedup = max([max(speedups) for speedups in data.values() if speedups])
    if max_speedup == 0: max_speedup = 1

    width = 800
    height = 500
    margin = 60

    # Chart area
    chart_width = width - 2 * margin
    chart_height = height - 2 * margin

    # SVG Elements
    svg = []
    svg.append(f'<svg width="{width}" height="{height}" xmlns="http://www.w3.org/2000/svg" style="background-color: white;">')
    svg.append(f'<text x="{width/2}" y="30" font-family="sans-serif" font-size="20" text-anchor="middle">CPU Thread Scaling</text>')

    # Axes
    svg.append(f'<line x1="{margin}" y1="{height-margin}" x2="{width-margin}" y2="{height-margin}" stroke="black" stroke-width="2"/>') # X-axis
    svg.append(f'<line x1="{margin}" y1="{margin}" x2="{margin}" y2="{height-margin}" stroke="black" stroke-width="2"/>') # Y-axis

    svg.append(f'<text x="{width/2}" y="{height-margin+40}" font-family="sans-serif" font-size="14" text-anchor="middle">Threads</text>')
    svg.append(f'<text x="{margin-40}" y="{height/2}" font-family="sans-serif" font-size="14" text-anchor="middle" transform="rotate(-90 {margin-40},{height/2})">Speedup (relative to 1 thread)</text>')

    # X-axis ticks (Threads)
    max_threads = max(thread_counts)
    for t in thread_counts:
        x = margin + (t / max_threads) * chart_width
        svg.append(f'<line x1="{x}" y1="{height-margin}" x2="{x}" y2="{height-margin+5}" stroke="black" stroke-width="1"/>')
        svg.append(f'<text x="{x}" y="{height-margin+20}" font-family="sans-serif" font-size="12" text-anchor="middle">{t}</text>')

    # Y-axis ticks (Speedup)
    y_ticks = int(max_speedup) + 1
    for y_val in range(y_ticks + 1):
        if y_val > max_speedup + 1: break
        if y_ticks > 10 and y_val % (y_ticks//5) != 0 and y_val != y_ticks: continue

        y = height - margin - (y_val / max_speedup) * chart_height
        svg.append(f'<line x1="{margin-5}" y1="{y}" x2="{margin}" y2="{y}" stroke="black" stroke-width="1"/>')
        svg.append(f'<text x="{margin-10}" y="{y+4}" font-family="sans-serif" font-size="12" text-anchor="end">{y_val}</text>')

        # Grid line
        svg.append(f'<line x1="{margin}" y1="{y}" x2="{width-margin}" y2="{y}" stroke="lightgray" stroke-width="1" stroke-dasharray="4"/>')

    # Plot lines
    colors = ['#1f77b4', '#ff7f0e', '#2ca02c', '#d62728', '#9467bd', '#8c564b', '#e377c2', '#7f7f7f', '#bcbd22', '#17becf']

    for idx, (packs, speedups) in enumerate(data.items()):
        color = colors[idx % len(colors)]

        points = []
        for i, t in enumerate(thread_counts):
            if i < len(speedups):
                x = margin + (t / max_threads) * chart_width
                y = height - margin - (speedups[i] / max_speedup) * chart_height
                points.append(f"{x},{y}")
                svg.append(f'<circle cx="{x}" cy="{y}" r="4" fill="{color}"/>')

        if len(points) > 1:
            path_d = f"M {points[0]} " + " L ".join(points[1:])
            svg.append(f'<path d="{path_d}" fill="none" stroke="{color}" stroke-width="2"/>')

        # Legend
        leg_y = margin + idx * 20
        svg.append(f'<line x1="{width - margin - 100}" y1="{leg_y}" x2="{width - margin - 80}" y2="{leg_y}" stroke="{color}" stroke-width="2"/>')
        svg.append(f'<circle cx="{width - margin - 90}" cy="{leg_y}" r="4" fill="{color}"/>')
        svg.append(f'<text x="{width - margin - 70}" y="{leg_y+4}" font-family="sans-serif" font-size="12">{packs} packs</text>')

    svg.append('</svg>')

    with open(filename, 'w') as f:
        f.write("\n".join(svg))

def main():
    pack_counts = [10000, 100000, 1000000, 5000000]
    thread_counts = [1, 2, 4, 8, 12, 24]

    results = {p: [] for p in pack_counts}
    speedups = {p: [] for p in pack_counts}

    print("Running benchmarks...")

    for packs in pack_counts:
        base_throughput = None
        for threads in thread_counts:
            tp = run_benchmark(packs, threads)
            results[packs].append(tp)

            if threads == 1:
                base_throughput = tp
                speedups[packs].append(1.0)
            else:
                if base_throughput and base_throughput > 0:
                    speedups[packs].append(tp / base_throughput)
                else:
                    speedups[packs].append(0.0)

    # Print Terminal ASCII Table
    print("\n" + "="*80)
    print(f"{'Pack Count':<12} | " + " | ".join([f"T={t:<6}" for t in thread_counts]))
    print("-" * 80)

    for packs in pack_counts:
        row_tp = []
        row_su = []
        for i, tp in enumerate(results[packs]):
            row_tp.append(f"{tp/1e6:5.2f}M")
            row_su.append(f"{speedups[packs][i]:5.2f}x")

        print(f"{packs:<12} | " + " | ".join([f"{tp:<8}" for tp in row_tp]) + " (Throughput)")
        print(f"{'':<12} | " + " | ".join([f"{su:<8}" for su in row_su]) + " (Speedup)")
        print("-" * 80)

    # Create Chart
    svg_filename = "scaling_plot.svg"
    if HAS_MATPLOTLIB:
        print("\nUsing matplotlib to generate chart...")
        plt.figure(figsize=(10, 6))
        for packs in pack_counts:
            plt.plot(thread_counts, speedups[packs], marker='o', label=f'{packs} packs')

        plt.xlabel('Threads')
        plt.ylabel('Speedup (relative to 1 thread)')
        plt.title('CPU Thread Scaling')
        plt.grid(True)
        plt.legend()
        plt.savefig(svg_filename)
        print(f"Saved chart to {svg_filename}")
    else:
        print("\nMatplotlib not found. Generating basic SVG chart...")
        generate_svg(speedups, thread_counts, svg_filename)
        print(f"Saved chart to {svg_filename}")

if __name__ == "__main__":
    main()
