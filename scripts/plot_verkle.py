import os
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker

widths = [
    2,
    4,
    32,
    64,
    256,
    512,
    1024
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

fig, ax1 = plt.subplots(figsize=(8, 6))
ax2 = ax1.twinx()

gas_price = 10 # in gwei
eth_price = 3500 # eth/usd

for width in widths:
    gas_data = []
    usd_data = []
    for size in sizes:
        table = pd.read_csv('./result/verkle_e2e/verkle_' + str(size) + ".csv")
        filtered_table = table[table["width"] == width]
        gas = filtered_table[filtered_table.keys()[3]]
        usd = gas * gas_price * eth_price * pow(10, -9)

        gas_data.append(gas)
        usd_data.append(usd)
    
    ax1.plot(sizes, gas_data, linestyle="solid", marker='o', label="k: " + str(width))
    ax2.plot(sizes, usd_data, linestyle="None")

ax1.set_ylim(ymin=0, ymax=1800000)
ax2.set_ylim(ymin=0,ymax=63)
ax1.ticklabel_format(axis='y', scilimits=[0, 0])
ax1.yaxis.set_major_formatter(ticker.FuncFormatter(lambda x, _: f'{x / 1e6:.2f}'))
ax1.yaxis.set_major_locator(ticker.MultipleLocator(250000))
ax2.yaxis.set_major_locator(ticker.MultipleLocator(5))
plt.text(0, 1.04, '1e6', ha='left', va='top', transform=ax1.transAxes)

ax1.set_xlabel('Vector length (Element size = 32 bytes)')
ax1.set_ylabel('Gas usage')
ax2.set_ylabel('Transaction fee (USD)')
ax1.set_xscale('log')
ax1.legend()

plt.show()
if not os.path.exists('./result/figures'):
    os.makedirs('./result/figures')
fig.savefig("./result/figures/verkle_e2e.pdf", transparent=True)
