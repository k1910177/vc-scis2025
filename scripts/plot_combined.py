import os
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import matplotlib.ticker as ticker

powers = [6,9,12]
colors = plt.rcParams['axes.prop_cycle'].by_key()['color']

fig, (ax_gas_top, ax_gas_bottom) = plt.subplots(2, figsize=(8, 6), gridspec_kw={'height_ratios': [1, 7]})
ax_usd = ax_gas_bottom.twinx()

gas_price = 10 # in gwei
eth_price = 3500 # eth/usd

v_csv = pd.read_csv('result/verkle_verify.csv')
m_csv = pd.read_csv('result/merkle_verify.csv')

m_color = 0
v_color = 0

for power in powers:
    v_color = m_color + 1

    v_table = v_csv[(v_csv["power"] == power)]
    m_table = m_csv[(m_csv["power"] == power)]
    
    width = v_table["width"].to_numpy(dtype='int')

    v_gas = v_table["gas"].to_numpy()
    m_gas = m_table["gas"].to_numpy()
    v_usd = v_gas * gas_price * eth_price * pow(10, -9)
    m_usd = m_gas * gas_price * eth_price * pow(10, -9)
    
    
    ax_gas_top.plot(width, m_gas, linestyle="solid", marker='o', label="Merkle Tree (n: 1e" + str(power) + ")", color=colors[m_color])
    ax_gas_bottom.plot(width, v_gas, linestyle="solid", marker='o', label="Verkle Tree (n: 1e" + str(power) + ")", color=colors[v_color])
    ax_gas_bottom.plot(width, m_gas, linestyle="solid", marker='o', label="Merkle Tree (n: 1e" + str(power) + ")", color=colors[m_color])
    ax_usd.plot(width, v_usd, linestyle="None", color=colors[v_color])
    ax_usd.plot(width, m_usd, linestyle="None", color=colors[m_color])

    m_color = v_color + 1

ax_gas_top.set_ylim(10_000_000, 32_000_000)
ax_gas_bottom.set_ylim(0, 3_200_000)
ax_usd.set_ylim(0, 110)

ax_gas_top.spines.bottom.set_visible(False)
ax_gas_bottom.spines.top.set_visible(False)
ax_usd.spines.top.set_visible(False)
ax_gas_top.xaxis.set_visible(False)
ax_gas_bottom.xaxis.tick_bottom()
ax_gas_top.tick_params(labeltop=False) 

ax_gas_bottom.set_xscale('log', base=2)
ax_gas_top.set_xscale('log', base=2)
ax_usd.set_xscale('log', base=2)

# ax_bottom_usd.yaxis.set_label_coords(0.95, 0.5, transform=fig.transFigure)
formatter = ticker.FuncFormatter(lambda x, _: f'{x / 1e6:.2f}')
ax_gas_top.yaxis.set_major_formatter(formatter)
ax_gas_bottom.yaxis.set_major_formatter(formatter)
ax_usd.yaxis.set_major_locator(ticker.MultipleLocator(10))
plt.text(0, 1.3, '1e6', ha='left', va='top', transform=ax_gas_top.transAxes)

x_ticks = [2**n for n in range(1,11)]  # Generate ticks as 2^n
ax_gas_bottom.set_xticks(x_ticks)  # Set the ticks
ax_gas_bottom.set_xticklabels([f"$2^{{{n}}}$" for n in range(1,11)])  

ax_gas_bottom.set_xlabel(f"Branching factor $k$")
ax_gas_bottom.set_ylabel('Gas usage')
ax_usd.set_ylabel('Transaction fee (USD)')

ax_gas_bottom.legend()

ax_gas_bottom.yaxis.set_label_coords(0.06, 0.5, transform=fig.transFigure)
ax_usd.yaxis.set_label_coords(0.95, 0.5, transform=fig.transFigure)

d = .5  # proportion of vertical to horizontal extent of the slanted line
kwargs = dict(marker=[(-1, -d), (1, d)], markersize=12,
              linestyle="none", color='k', mec='k', mew=1, clip_on=False)
ax_gas_top.plot([0, 1], [0, 0], transform=ax_gas_top.transAxes, **kwargs)
ax_gas_bottom.plot([0, 1], [1, 1], transform=ax_gas_bottom.transAxes, **kwargs)

plt.show()
if not os.path.exists('./result/figures'):
    os.makedirs('./result/figures')
fig.savefig("./result/figures/verify_combined.pdf",  transparent=True)
