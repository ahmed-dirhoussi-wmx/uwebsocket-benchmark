import os
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt
from dotenv import load_dotenv

from PIL import Image

sns.set_style("whitegrid")


def latency_markdown(stats, batch_size, n_batchs, waits, md_dir):
    stats_df = pd.DataFrame(stats)
    path = os.path.join(
        md_dir, f"latency_nclients_b{batch_size}_n{n_batchs}_w{waits}.md"
    )
    with open(path, "w") as f:
        f.write(stats_df.to_markdown())


def concat_imgs_nc(plots_dir, nclients, batch_size, n_batchs, waits):
    names = ["server_latency", "p99", "p90", "server_latency_dist"]
    for base in names:
        imgs_path = []
        for nc in nclients:
            plot_path = os.path.join(
                plots_dir, f"{base}_c{nc}_b{batch_size}_n{n_batchs}_w{waits}.png"
            )
            if os.path.exists(plot_path):
                print(f"Found {plot_path}")
                imgs_path.append(plot_path)

        images = [Image.open(x) for x in imgs_path]
        widths, heights = zip(*(i.size for i in images))
        total_width = sum(widths)
        max_height = max(heights)
        new_im = Image.new("RGB", (total_width, max_height))
        x_offset = 0
        for im in images:
            new_im.paste(im, (x_offset, 0))
            x_offset += im.size[0]

        agg_path = os.path.join(plots_dir, f"results_agg/{base}_nclients.jpg")
        new_im.save(agg_path)


def plot_latency_quantile(
    server_latency, quantile, n_clients, batch_size, n_batchs, wait, save_dir
):
    lquantile = server_latency.quantile(float(quantile / 100), interpolation="nearest")
    latencies = server_latency[server_latency > lquantile]
    plt.figure(figsize=(10, 6))
    sns.histplot(latencies)
    plt.title(
        f"Distribution of Latencies above p{quantile}: {lquantile}ms.\n{n_clients} client, {batch_size} batch_size, {n_batchs} batches, {wait}ms wait"
    )
    plt.xlabel("Latency")
    plt.ylabel("Frequency")
    plt.savefig(
        f"./{save_dir}/p{quantile}_c{n_clients}_b{batch_size}_n{n_batchs}_w{wait}.png"
    )


def plot_latency_dist(server_latency, n_clients, batch_size, n_batchs, wait, save_dir):
    description = server_latency.describe().to_string()
    plt.figure(figsize=(10, 6))
    sns.histplot(server_latency)
    plt.title(
        f"Server Latency distribution for {n_clients} client, {batch_size} batch_size, {n_batchs} nbatch, {wait}ms wait "
    )
    plt.text(
        0.5,
        0.95,
        description,
        transform=plt.gca().transAxes,
        fontsize=10,
        verticalalignment="top",
        horizontalalignment="center",
    )
    plt.savefig(
        f"./{save_dir}/server_latency_dist_c{n_clients}_b{batch_size}_n{n_batchs}_w{wait}.png"
    )


def plot_latency(server_latency, n_clients, batch_size, n_batchs, wait, save_dir):
    plt.figure(figsize=(10, 6))
    server_latency.rolling(100).mean().plot()
    plt.title(
        f"Server Latency Over Time (1000-rolling avg). \n{n_clients} client, {batch_size} batch_size, {n_batchs} batches, {wait}ms wait"
    )
    plt.xlabel("Time")
    plt.ylabel("Latency")
    plt.savefig(
        f"./{save_dir}/server_latency_c{n_clients}_b{batch_size}_n{n_batchs}_w{wait}.png"
    )


def plot_results(n_clients, batch_size, n_batchs, wait, csv_path, plots_dir, stats):
    if not os.path.exists(csv_path):
        print(f"{csv_path} experiment doesnt exist")
        return

    df = pd.read_csv(csv_path, header=0)
    df["timestamp"] = pd.to_datetime(df["timestamp"], unit="ms")
    df["client_created_at"] = pd.to_datetime(df["client_created_at"], unit="ms")
    df["client_timestamp"] = pd.to_datetime(df["client_created_at"], unit="ms")
    df = df.set_index("timestamp")

    server_latency = df["server_latency"]
    # TODO : add the roundtrip
    stats[f"{n_clients} clients"] = {
        "mean_latency": int(server_latency.mean()),
        "max_latency": server_latency.max(),
        "min_latency": server_latency.min(),
        "p50_latency": server_latency.median(),
        "p90_latency": server_latency.quantile(float(0.9), interpolation="nearest"),
        "p99_latency": server_latency.quantile(float(0.99), interpolation="nearest"),
    }

    plot_latency(server_latency, n_clients, batch_size, n_batchs, wait, plots_dir)
    plot_latency_dist(server_latency, n_clients, batch_size, n_batchs, wait, plots_dir)
    plot_latency_quantile(
        server_latency, 90, n_clients, batch_size, n_batchs, wait, plots_dir
    )
    plot_latency_quantile(
        server_latency, 99, n_clients, batch_size, n_batchs, wait, plots_dir
    )
    return server_latency


if __name__ == "__main__":
    nclients = [1000, 3000, 5000, 10000]
    batch_sizes = [1]
    waits = [1000]
    n_batchs = 100

    # Load environment variables from .env file
    load_dotenv("../.env")
    results_dir = os.path.basename(os.getenv("RESULTS_VOLUME"))

    # Setup the results directoriesk
    plots_dir = f"plots/{results_dir}"
    md_dir = f"markdown/{results_dir}"
    # Create a local directory to save the plots and aggregates
    print(f"Creating {plots_dir}...")
    os.makedirs(os.path.join(plots_dir, "results_agg"), exist_ok=True)
    print(f"Creating {md_dir}...")
    os.makedirs(md_dir, exist_ok=True)

    stats_server_latencies = {}
    # For each combination generate plots
    for nc in nclients:
        for b in batch_sizes:
            for w in waits:
                csv_path = (
                    f"../results/{results_dir}/result_c{nc}_b{b}_n{n_batchs}_w{w}.csv"
                )
                print(f"Plotting : {nc}clients {b}batch_size {w}wait")
                plot_results(
                    n_clients=nc,
                    batch_size=b,
                    n_batchs=n_batchs,
                    wait=w,
                    csv_path=csv_path,
                    plots_dir=plots_dir,
                    stats=stats_server_latencies,
                )

    # Combine plots for different nb clients
    print("Aggregating results...")
    concat_imgs_nc(plots_dir, nclients, batch_sizes[0], n_batchs, waits[0])

    # Summary statics to markdown
    print(f"Generating latency summary statistics for nclients : {nclients}")
    latency_markdown(stats_server_latencies, batch_sizes[0], n_batchs, waits[0], md_dir)
