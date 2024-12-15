import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import os

widths = [
    2,
    4,
    32,
    64,
    256,
]

sizes = [
    1000, 
    5000,
    10_000, 
    50_000,
    100_000, 
    500_000,
    1_000_000,
    5_000_000,
]

fig, (ax_top, ax_bottom) = plt.subplots(2, figsize=(8, 6), gridspec_kw={'height_ratios': [1, 6]})
ax_top_usd = ax_top.twinx()
ax_bottom_usd = ax_bottom.twinx()

gas_price = 10 # in gwei
eth_price = 3500 # eth/usd

for width in widths:
    gas_data = []
    usd_data = []
    for size in sizes:
        table = pd.read_csv('result/merkle_e2e/merkle_' + str(size) + ".csv")
        filtered_table = table[table["width"] == width]
        gas = filtered_table[filtered_table.keys()[3]]
        usd = gas * gas_price * eth_price * pow(10, -9)

        gas_data.append(gas)
        usd_data.append(usd)

    ax_top.plot(sizes, gas_data, linestyle="solid", marker='o', label="k: " + str(width))
    ax_bottom.plot(sizes, gas_data, linestyle="solid", marker='o', label="k: " + str(width))
    ax_top_usd.plot(sizes, usd_data, linestyle="None")
    ax_bottom_usd.plot(sizes, usd_data, linestyle="None")

ax_top.set_ylim(8_000_000, 23_000_000)
ax_top_usd.set_ylim(300, 800)
ax_bottom.set_ylim(0, 500_000)
ax_bottom_usd.set_ylim(0, 17.5)

formatter = ticker.FuncFormatter(lambda x, _: f'{x / 1e6:.2f}')
ax_top.yaxis.set_major_formatter(formatter)
ax_bottom.yaxis.set_major_formatter(formatter)
ax_bottom.yaxis.set_major_locator(ticker.MultipleLocator(50_000))
ax_bottom_usd.yaxis.set_major_locator(ticker.MultipleLocator(1))
# ax_top_usd.yaxis.set_major_locator(ticker.MultipleLocator(250))

ax_top.spines.bottom.set_visible(False)
ax_bottom.spines.top.set_visible(False)
ax_top_usd.spines.bottom.set_visible(False)
ax_bottom_usd.spines.top.set_visible(False)
ax_top.xaxis.set_visible(False)
ax_top.tick_params(labeltop=False)  # don't put tick labels at the top
ax_top_usd.tick_params(labeltop=False)  # don't put tick labels at the top
ax_bottom.xaxis.tick_bottom()
ax_bottom_usd.xaxis.tick_bottom()

ax_bottom.set_xlabel('Vector length (Element size = 32 bytes)')
ax_bottom.set_ylabel('Gas usage')
ax_bottom_usd.set_ylabel('Transaction fee (USD)')
ax_bottom.set_xscale('log')
ax_top.set_xscale('log')
ax_bottom.legend(loc='upper left')

plt.text(0, 1.3, '1e6', ha='left', va='top', transform=ax_top.transAxes)

ax_bottom.yaxis.set_label_coords(0.06, 0.5, transform=fig.transFigure)
ax_bottom_usd.yaxis.set_label_coords(0.95, 0.5, transform=fig.transFigure)

d = .5  # proportion of vertical to horizontal extent of the slanted line
kwargs = dict(marker=[(-1, -d), (1, d)], markersize=12,
              linestyle="none", color='k', mec='k', mew=1, clip_on=False)
ax_top.plot([0, 1], [0, 0], transform=ax_top.transAxes, **kwargs)
ax_bottom.plot([0, 1], [1, 1], transform=ax_bottom.transAxes, **kwargs)

plt.show()
if not os.path.exists('./result/figures'):
    os.makedirs('./result/figures')
fig.savefig("./result/figures/merkle_e2e.pdf", transparent=True)
